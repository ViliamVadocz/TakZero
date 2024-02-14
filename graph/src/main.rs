use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{BufRead, BufReader},
    path::Path,
};

use charming::{
    component::{Axis, Grid, Legend, Title},
    element::Symbol,
    series::Line,
    theme::Theme,
    Chart,
    HtmlRenderer,
};
use takzero::{network::net4::Env, search::env::Environment, target::Replay};

fn main() {
    let chart = Chart::new()
        .title(
            Title::new()
                .text("Ratio of Unique Positions to All Positions Seen During Training")
                .subtext("Accounting for Symmetries")
                .left("center")
                .top(0),
        )
        .x_axis(Axis::new().name("Games"))
        .y_axis(Axis::new().name("Ratio"))
        .grid(Grid::new())
        .legend(
            Legend::new()
                .data(vec!["Baseline", "Exploration"])
                .right(10),
        )
        .series(
            Line::new()
                .data(get_unique_positions("baseline-replays.txt"))
                .name("Baseline")
                .symbol(Symbol::None),
        )
        .series(
            Line::new()
                .data(get_unique_positions("exploration-replays.txt"))
                .name("Exploration")
                .symbol(Symbol::None),
        );

    let mut renderer = HtmlRenderer::new("graph", 1000, 600).theme(Theme::Infographic);
    renderer.save(&chart, "graph.html").unwrap();
}

fn get_unique_positions(path: impl AsRef<Path>) -> Vec<Vec<f64>> {
    let file = OpenOptions::new().read(true).open(path).unwrap();
    let mut positions: HashMap<_, u64> = HashMap::new();
    let mut points = Vec::with_capacity(4096);
    for (i, line) in BufReader::new(file).lines().enumerate() {
        if i % 2_000 == 0 {
            points.push(vec![
                i as f64,
                if i == 0 {
                    1.0
                } else {
                    positions.keys().len() as f64 / positions.values().sum::<u64>() as f64
                },
            ]);
        }
        let Ok(replay): Result<Replay<Env>, _> = line.unwrap().parse() else {
            println!("skipping line {i}");
            continue;
        };
        let mut env = replay.env;
        *positions.entry(env.clone().canonical()).or_default() += 1;
        for action in replay.actions {
            env.step(action);
            *positions.entry(env.clone().canonical()).or_default() += 1;
        }
    }
    println!("unique positions: {}", positions.keys().len());
    println!("total positions: {}", positions.values().sum::<u64>());
    points
}
