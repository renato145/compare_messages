use crate::TestResult;
use anyhow::Result;
use itertools::Itertools;
use plotters::prelude::*;

const OUT_FILE_NAME: &'static str = "results.svg";

pub fn plot(results: Vec<TestResult>) -> Result<()> {
    let root = SVGBackend::new(OUT_FILE_NAME, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let (upper, lower) = root.split_vertically(750);

    lower.titled(
        "Some text on lower",
        ("sans-serif", 10).into_font().color(&BLACK.mix(0.5)),
    )?;

    let mut chart = ChartBuilder::on(&upper)
        .caption("Some title", ("sans-serif", (5).percent_height()))
        .set_label_area_size(LabelAreaPosition::Left, (8).percent())
        .set_label_area_size(LabelAreaPosition::Bottom, (4).percent())
        .margin((1).percent())
        .build_cartesian_2d(0..1000, 0..1000)?;

    chart
        .configure_mesh()
        .x_desc("Array size in message")
        .y_desc("Time")
        .draw()?;

    let data = results
        .into_iter()
        .map(|x| (x.title.clone(), x))
        .into_group_map();

    for (idx, (title, result)) in data.into_iter().enumerate() {
        let color = Palette99::pick(idx).mix(0.9);
        chart.draw_series(LineSeries::new(
            get_plot_data(result),
            color.stroke_width(3),
        ))?;
    }

    Ok(())
}

/// Extracts x & y coordinates from `data`
fn get_plot_data(data: Vec<TestResult>) -> Vec<(f64, f64)> {
    data.into_iter()
        .map(|x| (x.num_elements, x.elapsed.as_millis()))
        .collect::<Vec<_>>();
		todo!()
}
