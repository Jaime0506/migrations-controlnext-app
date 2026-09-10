use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;
use futures::stream::{self, StreamExt};
use std::sync::Arc;
use tauri::Emitter;

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConnection {
    pub id: String,
    #[serde(rename = "type")]
    pub db_type: Option<String>,
    pub host: Option<String>,
    pub db: Option<String>,
    pub schema: Option<String>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionResult {
    pub connection_id: String,
    pub success: bool,
    pub message: String,
}

/// Construye la connection string para PostgreSQL
fn build_connection_string(conn: &DatabaseConnection) -> Result<String, String> {
    let host = conn.host.as_deref().unwrap_or("localhost");
    let port = conn.port.unwrap_or(5432);
    let user = conn.user.as_deref().ok_or("User is required")?;
    let password = conn.password.as_deref().ok_or("Password is required")?;
    let db = conn.db.as_deref().ok_or("Database name is required")?;

    Ok(format!(
        "host={} port={} user={} password={} dbname={}",
        host, port, user, password, db
    ))
}

/// Ejecuta SQL en una conexión específica
async fn execute_on_connection(
    conn: &DatabaseConnection,
    sql: &str,
) -> Result<ExecutionResult, ExecutionResult> {
    let connection_id = conn.id.clone();
    
    // Construir connection string
    let connection_string = match build_connection_string(conn) {
        Ok(cs) => cs,
        Err(e) => {
            let error_msg = format!("Error construyendo connection string: {}", e);
            return Err(ExecutionResult {
                connection_id: connection_id.clone(),
                success: false,
                message: error_msg,
            });
        }
    };

    // Conectar a la base de datos con un timeout extendido de 20 segundos
    // para evitar falsos negativos por congestión temporal de sockets o DNS
    let connect_future = tokio_postgres::connect(&connection_string, NoTls);
    let (mut client, connection) = match tokio::time::timeout(std::time::Duration::from_secs(20), connect_future).await {
        Ok(Ok((c, conn))) => (c, conn),
        Ok(Err(e)) => {
            let error_str = e.to_string();
            let error_msg = if error_str.contains("password authentication failed") {
                "Error de autenticación: Usuario o contraseña incorrectos".to_string()
            } else if error_str.contains("connection refused") {
                "Error de conexión: Conexión rechazada por el servidor (verifica host y puerto)".to_string()
            } else if error_str.contains("does not exist") {
                "Error: La base de datos no existe".to_string()
            } else {
                format!("Error conectando a la base de datos: {}", e)
            };
            
            return Err(ExecutionResult {
                connection_id: connection_id.clone(),
                success: false,
                message: error_msg,
            });
        }
        Err(_) => {
            let error_msg = "Error de conexión: Tiempo de espera agotado (Timeout > 20s)".to_string();
            return Err(ExecutionResult {
                connection_id: connection_id.clone(),
                success: false,
                message: error_msg,
            });
        }
    };

    // Ejecutar la tarea de conexión en segundo plano
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Error en el stream de conexión: {}", e);
        }
    });

    // Establecer el schema si está especificado
    if let Some(schema) = &conn.schema {
        let set_schema_sql = format!("SET search_path TO {}", schema);
        if let Err(e) = client.execute(&set_schema_sql, &[]).await {
            let error_msg = format!("Error estableciendo schema '{}': {}", schema, e);
            return Err(ExecutionResult {
                connection_id: connection_id.clone(),
                success: false,
                message: error_msg,
            });
        }
    }

    let sql_trimmed = sql.trim();
    let sql_upper = sql_trimmed.to_uppercase();
    
    // DDL (Data Definition Language)
    let has_ddl = sql_upper.contains("CREATE ") 
        || sql_upper.contains("ALTER ") 
        || sql_upper.contains("DROP ")
        || sql_upper.contains("TRUNCATE ")
        || sql_upper.contains("COMMENT ON");
    
    // DCL (Data Control Language)
    let has_dcl = sql_upper.contains("GRANT ") 
        || sql_upper.contains("REVOKE ");
    
    // Funciones y procedimientos
    let has_functions = sql_upper.contains("CREATE FUNCTION")
        || sql_upper.contains("CREATE OR REPLACE FUNCTION")
        || sql_upper.contains("CREATE PROCEDURE")
        || sql_upper.contains("CREATE OR REPLACE PROCEDURE");
    
    let has_create_table_as = sql_upper.contains("CREATE TABLE") && sql_upper.contains("AS SELECT");
    let semicolon_count = sql_trimmed.matches(';').count();
    let has_multiple_statements = semicolon_count > 1;
    
    let use_batch = has_ddl || has_dcl || has_functions || has_create_table_as || has_multiple_statements;
    
    if use_batch {
        let statement_type = if has_ddl { "DDL" }
            else if has_dcl { "DCL" }
            else if has_functions { "FUNCTION/PROCEDURE" }
            else if has_create_table_as { "CREATE TABLE AS SELECT" }
            else { "MÚLTIPLES STATEMENTS" };
        
        let transaction = match client.transaction().await {
            Ok(txn) => txn,
            Err(e) => {
                let error_msg = format!("Error iniciando transacción: {}", e);
                return Err(ExecutionResult {
                    connection_id: connection_id.clone(),
                    success: false,
                    message: error_msg,
                });
            }
        };
        
        if let Some(schema) = &conn.schema {
            let set_schema_sql = format!("SET search_path TO {}", schema);
            if let Err(e) = transaction.execute(&set_schema_sql, &[]).await {
                let error_msg = format!("Error estableciendo schema '{}' en transacción: {}", schema, e);
                return Err(ExecutionResult {
                    connection_id: connection_id.clone(),
                    success: false,
                    message: error_msg,
                });
            }
        }
        
        match transaction.batch_execute(sql).await {
            Ok(_) => {
                match transaction.commit().await {
                    Ok(_) => {
                        let success_msg = format!("SQL ejecutado exitosamente ({})", statement_type);
                        Ok(ExecutionResult {
                            connection_id: connection_id.clone(),
                            success: true,
                            message: success_msg,
                        })
                    }
                    Err(e) => {
                        let error_msg = format!("Error haciendo commit: {}", e);
                        Err(ExecutionResult {
                            connection_id: connection_id.clone(),
                            success: false,
                            message: error_msg,
                        })
                    }
                }
            }
            Err(e) => {
                let error_msg = if let Some(db_error) = e.as_db_error() {
                    format!(
                        "Error de base de datos: {} (Código: {})",
                        db_error.message(),
                        db_error.code().code()
                    )
                } else {
                    format!("Error ejecutando SQL: {}", e)
                };
                
                Err(ExecutionResult {
                    connection_id: connection_id.clone(),
                    success: false,
                    message: error_msg,
                })
            }
        }
    } else {
        match client.execute(sql, &[]).await {
            Ok(rows_affected) => {
                let success_msg = format!("SQL ejecutado exitosamente. Filas afectadas: {}", rows_affected);
                Ok(ExecutionResult {
                    connection_id: connection_id.clone(),
                    success: true,
                    message: success_msg,
                })
            }
            Err(e) => {
                let error_msg = if let Some(db_error) = e.as_db_error() {
                    format!(
                        "Error de base de datos: {} (Código: {})",
                        db_error.message(),
                        db_error.code().code()
                    )
                } else {
                    format!("Error ejecutando SQL: {}", e)
                };
                
                Err(ExecutionResult {
                    connection_id: connection_id.clone(),
                    success: false,
                    message: error_msg,
                })
            }
        }
    }
}

#[tauri::command]
pub async fn execute_sql(
    app: tauri::AppHandle,
    sql: String,
    connections: Vec<DatabaseConnection>,
) -> Result<Vec<ExecutionResult>, String> {
    let total_count = connections.len();
    println!("=== Iniciando ejecución de SQL en {} conexiones ===", total_count);

    // Concurrencia óptima de 12 tareas en paralelo:
    // Evita la saturación del stack de red de Windows (sockets TCP half-open, DNS throttling)
    // y no colapsa los límites de conexión de PostgreSQL remoto.
    let concurrency_limit = 12;

    let sql_arc = Arc::new(sql);
    let (tx, mut rx) = tokio::sync::mpsc::channel::<ExecutionResult>(concurrency_limit * 2);

    // Tarea de emisión de progreso por lotes hacia Tauri frontend:
    // Agrupa resultados cada 120ms o al acumular 10 elementos,
    // evitando saturar el puente IPC y los re-renderizados en React.
    let app_clone = app.clone();
    let progress_task = tokio::spawn(async move {
        let mut batch = Vec::with_capacity(12);
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(120));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                res = rx.recv() => {
                    match res {
                        Some(item) => {
                            batch.push(item);
                            if batch.len() >= 10 {
                                let to_send = std::mem::take(&mut batch);
                                let _ = app_clone.emit("sql-execution-progress", &to_send);
                            }
                        }
                        None => {
                            // Canal cerrado al finalizar todos los workers
                            if !batch.is_empty() {
                                let _ = app_clone.emit("sql-execution-progress", &batch);
                            }
                            break;
                        }
                    }
                }
                _ = interval.tick() => {
                    if !batch.is_empty() {
                        let to_send = std::mem::take(&mut batch);
                        let _ = app_clone.emit("sql-execution-progress", &to_send);
                    }
                }
            }
        }
    });

    // Convertir a stream asíncrono con control de concurrencia
    let results: Vec<ExecutionResult> = stream::iter(connections.into_iter())
        .map(|conn| {
            let sql_clone = Arc::clone(&sql_arc);
            let tx_clone = tx.clone();
            async move {
                let res = match execute_on_connection(&conn, &sql_clone).await {
                    Ok(result) => result,
                    Err(result) => result,
                };
                
                // Enviar al canal de progreso en lotes
                let _ = tx_clone.send(res.clone()).await;
                res
            }
        })
        .buffer_unordered(concurrency_limit)
        .collect()
        .await;

    // Cerrar el transmisor original y esperar que la tarea de emisión termine
    drop(tx);
    let _ = progress_task.await;

    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;
    
    println!("=== Resumen de ejecución ===");
    println!("Total procesadas: {} | Exitosas: {} | Fallidas: {}", results.len(), successful, failed);

    Ok(results)
}