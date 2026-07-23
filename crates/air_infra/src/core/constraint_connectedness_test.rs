use std::collections::HashSet;
use std::sync::{LazyLock, Mutex};

use air_common::UseOrYield;

use crate::core::air_body::*;
use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;

static EXCLUDED_AIR_FNS: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

pub fn exclude<E, I, O>(air_fn: &dyn AirFn<ExtIn = E, In = I, Out = O>)
where
    E: ExtTable,
    I: AirVar,
    O: AirVar,
{
    EXCLUDED_AIR_FNS.lock().unwrap().insert(air_fn.name());
}

// Represents an undirected hypergraph (that is, an edge can connect more than
// two vertices)
#[derive(Default)]
struct Graph {
    vertices: HashSet<String>,
    edges: Vec<HashSet<String>>,
}

impl Graph {
    fn is_connected(&self) -> bool {
        if self.vertices.is_empty() {
            return true;
        }

        // A connected component of the graph
        let mut component = HashSet::<String>::new();

        // Start with one arbitrary vertex
        component.insert(self.vertices.iter().next().unwrap().clone());

        let mut changed = true;
        while changed {
            changed = false;
            for edge in self.edges.iter() {
                let in_component = edge.iter().any(|v| component.contains(v));
                if in_component {
                    // This edge is part of the connected component - add all its vertices to the
                    // component
                    for v in edge {
                        changed |= component.insert(v.clone());
                    }
                }
            }
        }

        // The graph is connected iff the component we built is the whole graph
        component.len() == self.vertices.len()
    }

    fn remove_vertex(&mut self, vertex: &str) {
        self.vertices.remove(vertex);
        for e in self.edges.iter_mut() {
            e.remove(vertex);
        }
    }
}

pub fn assert_constraint_graph_connected(entry: &AirFnEntry) {
    if EXCLUDED_AIR_FNS.lock().unwrap().contains(&entry.name) {
        return;
    }

    assert!(
        build_constraint_graph(entry).is_connected(),
        "Constraint graph for {} is not connected",
        entry.name
    );
}

/// Return all graph vertices (state cells and intermediate values) this expression depends on
fn expr_vertices(expr: &FeltExpr) -> HashSet<String> {
    let var_infos = expr.var_infos();
    let state_cells = var_infos.iter().filter_map(|vi| vi.get_state_cell_name());
    let intermediates =
        var_infos.iter().filter_map(|vi| vi.get_used_constraint_intermediate_name());

    state_cells.chain(intermediates).collect()
}

fn build_constraint_graph(entry: &AirFnEntry) -> Graph {
    let mut graph = Graph::default();

    // Add all state cells as vertices to catch unconstrained cells.
    graph.vertices.extend(entry.state.get_state_names());

    let constraint_components = entry.air_body.get_flattened_constraint_components();
    let mut multiplicity_vertices = HashSet::new();

    for component in constraint_components {
        match component {
            ConstraintComponent::Constraint(expr) => graph.edges.push(expr_vertices(&expr)),
            ConstraintComponent::Intermediate { name, value } => {
                let mut edge = expr_vertices(&value);
                edge.insert(name.clone());
                graph.vertices.insert(name.clone());
                graph.edges.push(edge);
            }
            ConstraintComponent::LookupTerm {
                relation_name,
                felts,
                use_or_yield,
                multiplicity,
            } => {
                // When we use a relation, we assume that the component that creates it is
                // responsible for connecting its felts with constraints, so we
                // create an edge that includes all the felts from the LookupTerm.
                // For chain relations we also need to handle yields to the chain. By the structure
                // of chain relations, each yield has a corresponding use with the same "chain id",
                // and each felt in the yielded tuple is connected by the chain
                // constraints to that use. This effectively connects all the felts
                // in the yielded tuple to each other, so in this case, too, we
                // create an edge that connects the whole tuple.
                //
                // To know whether a relation is a chain relation or not, we assume that
                // components only ever yield to a chain relation or to their own relation
                let is_chain_yield = use_or_yield == UseOrYield::Yield
                    && !entry.relation_names.contains(&relation_name);

                if use_or_yield == UseOrYield::Use || is_chain_yield {
                    // Don't count yield operations - the component that yields to a relation
                    // is the one responsible for connecting the relation felts by constraints.
                    let edge: HashSet<_> = felts.iter().flat_map(expr_vertices).collect();
                    graph.edges.push(edge)
                }

                multiplicity_vertices.extend(expr_vertices(&multiplicity));
            }
        }
    }

    // Check consistency - all vertices are listed
    for edge in graph.edges.iter() {
        for v in edge {
            assert!(graph.vertices.contains(v), "Edge {edge:?} contains unknown vertex {v}");
        }
    }

    // Values used as multiplicity are excluded from the connectedness testing
    for vertex in multiplicity_vertices {
        graph.remove_vertex(&vertex);
    }

    graph
}
