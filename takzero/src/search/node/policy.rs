use ordered_float::{NotNan, OrderedFloat};

use super::{super::env::Environment, Node};

impl<E: Environment> Node<E> {
    #[must_use]
    pub fn most_visited_count(&self) -> f32 {
        self.children
            .iter()
            .map(|(_, node)| node.visit_count)
            .max()
            .unwrap_or_default() as f32
    }

    /// Get the improved policy for this node.
    ///
    /// # Panics
    ///
    /// Panics if the evaluation is NaN.
    pub fn improved_policy(
        &self,
        #[cfg(not(feature = "baseline"))] beta: f32,
    ) -> impl Iterator<Item = f32> + '_ {
        let most_visited_count = self.most_visited_count();
        let p = self.children.iter().map(move |(_, node)| -> NotNan<f32> {
            let completed_value: NotNan<f32> = NotNan::new(
                if node.needs_initialization() {
                    self.evaluation
                } else {
                    node.evaluation.negate()
                }
                .into(),
            )
            .expect("completed value should not be NaN");
            sigma(
                completed_value,
                #[cfg(not(feature = "baseline"))]
                node.variance,
                #[cfg(not(feature = "baseline"))]
                beta,
                most_visited_count,
            ) + node.policy
        });

        // Softmax
        let max = p.clone().max().unwrap_or_default();
        let exp = p.map(move |x| (x - max).exp());
        let sum: f32 = exp.clone().sum();
        exp.map(move |x| x / sum)
    }

    /// Get index of child which maximizes the improved policy.
    #[allow(clippy::missing_panics_doc)]
    pub fn select_with_improved_policy(&mut self, beta: f32) -> usize {
        self.improved_policy(
            #[cfg(not(feature = "baseline"))]
            beta,
        )
        .zip(self.children.iter())
        .enumerate()
        // Prune only losing moves to preserve optimality.
        .filter(|(_, (_, (_, node)))| !node.evaluation.is_win())
        // Minimize mean-squared-error between visits and improved policy
        .max_by_key(|(_, (pi, (_, node)))| {
            OrderedFloat(pi - node.visit_count as f32 / ((self.visit_count + 1) as f32))
        })
        .map(|(i, _)| i)
        .expect("there should always be a child to simulate")
    }
}

#[must_use]
#[allow(clippy::suboptimal_flops)]
pub fn sigma(
    q: NotNan<f32>,
    #[cfg(not(feature = "baseline"))] variance: NotNan<f32>,
    #[cfg(not(feature = "baseline"))] beta: f32,
    visit_count: f32,
) -> NotNan<f32> {
    const C_VISIT: f32 = 50.0; // Paper used 50, but 30 solves tests
    const C_SCALE: f32 = 1.0; // Paper used 1, but 0.1 solves tests
    #[cfg(feature = "baseline")]
    return q * (C_VISIT + visit_count) * C_SCALE;
    #[cfg(not(feature = "baseline"))]
    return (q + variance.sqrt() * beta) * (C_VISIT + visit_count) * C_SCALE;
}
