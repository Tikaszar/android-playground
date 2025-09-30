//! Dependency graph for module resolution

use std::collections::{HashMap, HashSet, VecDeque};
use playground_api::Dependency;
use playground_core_types::{CoreResult, CoreError};

/// Dependency graph for tracking module dependencies
pub struct DependencyGraph {
    /// Map from module to its dependencies
    dependencies: HashMap<String, Vec<String>>,
    /// Map from module to modules that depend on it
    dependents: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    /// Create a new dependency graph
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    /// Add a module with its dependencies
    pub fn add_module(&mut self, name: &str, deps: &[Dependency]) -> CoreResult<()> {
        // Check for circular dependencies
        for dep in deps {
            if self.would_create_cycle(name, dep.name) {
                return Err(CoreError::ModuleLoadFailed(format!(
                    "Circular dependency detected: {} -> {}",
                    name,
                    dep.name
                )));
            }
        }

        // Add dependencies
        let dep_names: Vec<String> = deps.iter().map(|d| d.name.to_string()).collect();
        self.dependencies.insert(name.to_string(), dep_names.clone());

        // Update dependents
        for dep in dep_names {
            self.dependents
                .entry(dep)
                .or_insert_with(Vec::new)
                .push(name.to_string());
        }

        Ok(())
    }

    /// Remove a module from the graph
    pub fn remove_module(&mut self, name: &str) -> CoreResult<()> {
        // Remove from dependencies
        if let Some(deps) = self.dependencies.remove(name) {
            // Update dependents
            for dep in deps {
                if let Some(dependents) = self.dependents.get_mut(&dep) {
                    dependents.retain(|d| d != name);
                    if dependents.is_empty() {
                        self.dependents.remove(&dep);
                    }
                }
            }
        }

        // Remove as a dependency
        self.dependents.remove(name);

        Ok(())
    }

    /// Get modules that depend on the given module
    pub fn get_dependents(&self, name: &str) -> CoreResult<Vec<String>> {
        Ok(self.dependents
            .get(name)
            .cloned()
            .unwrap_or_default())
    }

    /// Get dependencies of a module
    pub fn get_dependencies(&self, name: &str) -> CoreResult<Vec<String>> {
        Ok(self.dependencies
            .get(name)
            .cloned()
            .unwrap_or_default())
    }

    /// Get topological load order for all modules
    pub fn topological_sort(&self) -> CoreResult<Vec<String>> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        let mut in_progress = HashSet::new();

        for module in self.dependencies.keys() {
            if !visited.contains(module) {
                self.dfs_visit(
                    module,
                    &mut visited,
                    &mut in_progress,
                    &mut stack
                )?;
            }
        }

        Ok(stack)
    }

    /// DFS visit for topological sort
    fn dfs_visit(
        &self,
        module: &str,
        visited: &mut HashSet<String>,
        in_progress: &mut HashSet<String>,
        stack: &mut Vec<String>
    ) -> CoreResult<()> {
        if in_progress.contains(module) {
            return Err(CoreError::ModuleLoadFailed(format!(
                "Circular dependency detected at module: {}",
                module
            )));
        }

        in_progress.insert(module.to_string());

        if let Some(deps) = self.dependencies.get(module) {
            for dep in deps {
                if !visited.contains(dep) {
                    self.dfs_visit(dep, visited, in_progress, stack)?;
                }
            }
        }

        in_progress.remove(module);
        visited.insert(module.to_string());
        stack.push(module.to_string());

        Ok(())
    }

    /// Check if adding a dependency would create a cycle
    fn would_create_cycle(&self, from: &str, to: &str) -> bool {
        // BFS to check if we can reach 'from' starting from 'to'
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back(to);
        visited.insert(to);

        while let Some(current) = queue.pop_front() {
            if current == from {
                return true;
            }

            if let Some(deps) = self.dependencies.get(current) {
                for dep in deps {
                    if !visited.contains(dep.as_str()) {
                        visited.insert(dep);
                        queue.push_back(dep);
                    }
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_graph() {
        let mut graph = DependencyGraph::new();

        // Add modules with no dependencies
        graph.add_module("core/types", &[]).unwrap();
        graph.add_module("core/ecs", &[
            Dependency {
                name: "core/types",
                version_req: "^1.0",
                features: &[],
            }
        ]).unwrap();

        // Check dependencies
        let deps = graph.get_dependencies("core/ecs").unwrap();
        assert_eq!(deps, vec!["core/types"]);

        // Check dependents
        let dependents = graph.get_dependents("core/types").unwrap();
        assert_eq!(dependents, vec!["core/ecs"]);

        // Topological sort
        let order = graph.topological_sort().unwrap();
        assert_eq!(order, vec!["core/types", "core/ecs"]);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut graph = DependencyGraph::new();

        graph.add_module("a", &[]).unwrap();
        graph.add_module("b", &[
            Dependency {
                name: "a",
                version_req: "^1.0",
                features: &[],
            }
        ]).unwrap();

        // This should fail due to circular dependency
        let result = graph.add_module("a", &[
            Dependency {
                name: "b",
                version_req: "^1.0",
                features: &[],
            }
        ]);

        assert!(result.is_err());
    }
}