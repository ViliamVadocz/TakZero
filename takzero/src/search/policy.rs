use float_ord::FloatOrd;

use super::{env::Environment, mcts::Node};

impl<E: Environment> Node<E> {
    #[must_use]
    pub fn most_visited_count(&self) -> f32 {
        self.children
            .iter()
            .map(|(_, node)| node.visit_count)
            .max()
            .unwrap_or_default() as f32
    }

    #[must_use]
    pub fn improved_policy(&self) -> Vec<f32> {
        let most_visited_count = self.most_visited_count();
        let p = self.children.iter().map(|(_, node)| {
            let completed_value = if node.needs_initialization() {
                self.evaluation
            } else {
                node.evaluation.negate()
            }
            .into();
            node.policy + sigma(completed_value, most_visited_count)
        });

        // Softmax
        let max = p.clone().max_by_key(|&x| FloatOrd(x)).unwrap_or_default();
        let exp = p.map(|x| (x - max).exp());
        let sum: f32 = exp.clone().sum();
        exp.map(|x| x / sum).collect()
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn select_with_improved_policy(&mut self) -> &mut (E::Action, Self) {
        self.improved_policy()
            .into_iter()
            .zip(self.children.iter_mut())
            // Prune only losing moves to preserve optimality.
            .filter(|(_, (_, node))| !node.evaluation.is_win())
            // Minimize mean-squared-error between visits and improved policy
            .max_by_key(|(pi, (_, node))| {
                FloatOrd(pi - node.visit_count as f32 / ((self.visit_count + 1) as f32))
            })
            .map(|(_, child)| child)
            .expect("There should always be a child to simulate.")
    }
}

#[must_use]
pub fn sigma(q: f32, visit_count: f32) -> f32 {
    const C_VISIT: f32 = 30.0; // Paper used 50
    const C_SCALE: f32 = 0.1; // Paper used 1
    (C_VISIT + visit_count) * C_SCALE * q
}
