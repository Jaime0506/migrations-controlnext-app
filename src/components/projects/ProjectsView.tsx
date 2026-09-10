import { useEffect, useState, useMemo } from "react";
import { Plus, Search, FolderKanban, Database } from "lucide-react";
import { ProjectData } from "@/hooks/useProject";
import { useProjectService } from "@/hooks/useProjectService";
import ProjectCard from "./ProjectCard";
import ProjectCreateModal from "./ProjectCreateModal";

interface ProjectsViewProps {
  onProjectClick?: (project: ProjectData) => void;
}

export default function ProjectsView({ onProjectClick }: ProjectsViewProps) {
  const { getProjects } = useProjectService();
  const [projects, setProjects] = useState<ProjectData[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  const fetchProjects = () => {
    setIsLoading(true);
    getProjects((data: ProjectData[]) => {
      setProjects(data || []);
      setIsLoading(false);
    });
  };

  useEffect(() => {
    fetchProjects();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const filteredProjects = useMemo(() => {
    if (!searchQuery.trim()) return projects;
    const query = searchQuery.toLowerCase();
    return projects.filter(
      (p) =>
        p.name.toLowerCase().includes(query) ||
        (p.description && p.description.toLowerCase().includes(query)) ||
        (p.tags && p.tags.some((t) => t.toLowerCase().includes(query)))
    );
  }, [projects, searchQuery]);

  return (
    <div className="flex w-full h-full max-w-6xl mx-auto flex-col gap-6 p-6 sm:p-8 bg-surface-1/80 backdrop-blur-2xl border border-surface-border rounded-2xl shadow-2xl overflow-hidden">
      {/* Header del Hub */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-6 border-b border-surface-border shrink-0">
        <div className="space-y-1">
          <div className="flex items-center gap-2">
            <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-[11px] font-mono bg-cerulean-500/10 text-cerulean-300 border border-cerulean-500/20">
              <Database className="size-3" />
              Tenant Forge v2
            </span>
            <span className="text-xs text-muted-foreground">·</span>
            <span className="text-xs text-muted-foreground">
              {projects.length} {projects.length === 1 ? "proyecto registrado" : "proyectos registrados"}
            </span>
          </div>
          <h1 className="text-2xl font-bold text-foreground tracking-tight">
            Gestor de Proyectos & Bases de Datos
          </h1>
          <p className="text-xs text-muted-foreground">
            Selecciona un espacio de trabajo para administrar migraciones y sentencias SQL
          </p>
        </div>

        {/* Acciones principales */}
        <div className="flex items-center gap-3">
          <div className="relative w-full sm:w-64">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Buscar proyecto o tag..."
              className="w-full pl-9 pr-3 py-1.5 text-xs rounded-lg bg-surface-base border border-surface-border focus:border-cerulean-500 focus:outline-none text-foreground placeholder:text-muted-foreground/60 transition-colors"
            />
          </div>

          <button
            type="button"
            onClick={() => setIsCreateModalOpen(true)}
            className="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-lg text-xs font-semibold bg-cerulean-500 hover:bg-cerulean-400 text-surface-base transition-all duration-150 cursor-pointer shrink-0 shadow-[0_0_15px_rgba(8,191,247,0.25)]"
          >
            <Plus className="size-3.5" />
            <span>Nuevo Proyecto</span>
          </button>
        </div>
      </div>

      {/* Lista de Proyectos en Grid */}
      <div className="flex-1 overflow-y-auto pr-1">
        {isLoading ? (
          <div className="flex flex-col items-center justify-center py-20 text-muted-foreground">
            <div className="size-6 rounded-full border-2 border-cerulean-400 border-t-transparent animate-spin mb-3" />
            <p className="text-xs">Cargando proyectos...</p>
          </div>
        ) : filteredProjects.length > 0 ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {filteredProjects.map((project) => (
              <ProjectCard
                key={project.id}
                project={project}
                onClick={() => onProjectClick?.(project)}
              />
            ))}
          </div>
        ) : searchQuery.trim() ? (
          <div className="flex flex-col items-center justify-center py-16 text-center">
            <div className="size-12 rounded-full bg-surface-2 flex items-center justify-center text-muted-foreground mb-3">
              <Search className="size-5" />
            </div>
            <h3 className="text-sm font-semibold text-foreground">Sin resultados</h3>
            <p className="text-xs text-muted-foreground mt-1 mb-4">
              No se encontraron proyectos que coincidan con "{searchQuery}"
            </p>
            <button
              type="button"
              onClick={() => setSearchQuery("")}
              className="text-xs text-cerulean-300 hover:underline cursor-pointer"
            >
              Limpiar búsqueda
            </button>
          </div>
        ) : (
          /* Empty State */
          <div className="flex flex-col items-center justify-center py-16 border border-dashed border-surface-border rounded-xl bg-surface-base/40 text-center">
            <div className="size-12 rounded-xl bg-cerulean-500/10 border border-cerulean-500/20 flex items-center justify-center text-cerulean-400 mb-3">
              <FolderKanban className="size-6" />
            </div>
            <h3 className="text-sm font-semibold text-foreground">
              Aún no tienes proyectos configurados
            </h3>
            <p className="text-xs text-muted-foreground max-w-sm mt-1 mb-4">
              Crea tu primer proyecto para vincular conexiones `.env` y ejecutar consultas SQL en múltiples tenants.
            </p>
            <button
              type="button"
              onClick={() => setIsCreateModalOpen(true)}
              className="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-semibold bg-cerulean-500 hover:bg-cerulean-400 text-surface-base transition-all duration-150 cursor-pointer shadow-lg"
            >
              <Plus className="size-4" />
              <span>Crear mi primer proyecto</span>
            </button>
          </div>
        )}
      </div>

      {/* Modal de Creación */}
      <ProjectCreateModal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        onProjectCreated={fetchProjects}
      />
    </div>
  );
}
