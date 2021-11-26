use std::collections::HashMap;

use crate::TestResult;
use anyhow::Result;
use itertools::Itertools;
use plotters::prelude::*;

const OUT_FILE_NAME: &str = "results.svg";

pub fn plot(results: Vec<TestResult>) -> Result<()> {
    let root = SVGBackend::new(OUT_FILE_NAME, (800, 400)).into_drawing_area();
    root.fill(&WHITE)?;

    let bar_width = 50;
    let group_pad = 15;

    let title_map = results
        .iter()
        .map(|x| x.title.clone())
        .unique()
        .enumerate()
        .map(|(i, x)| (x, i))
        .collect::<HashMap<_, _>>();
    let x_map = results
        .iter()
        .map(|x| x.num_elements)
        .unique()
        .enumerate()
        .map(|(i, x)| (x, i))
        .collect::<HashMap<_, _>>();
    let group_width = bar_width * title_map.len() + group_pad * 2;
    let x_range = 0..(group_width * x_map.len());
    let y_range = {
        let (a, b) = results
            .iter()
            .map(|x| x.elapsed.as_millis())
            .minmax()
            .into_option()
            .expect("Missing data.");
        a..(b + 1)
    };

    let n_tests = (&results[0]).n_tests;
    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("Message comparisons ({} messages)", n_tests),
            ("sans-serif", (5).percent_height()),
        )
        .set_label_area_size(LabelAreaPosition::Left, (12).percent())
        .set_label_area_size(LabelAreaPosition::Bottom, (8).percent())
        .margin((1).percent())
        .build_cartesian_2d(x_range.clone(), y_range.clone())?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .x_desc("Array size in message")
        .y_desc("Time (ms)")
        .axis_desc_style(("sans-serif", 18).into_font())
        .disable_x_axis()
        .draw()?;

    let vlines = (1..x_range.len()-1).map(|x| {
        let x = x * group_width;
        PathElement::new(vec![(x, y_range.start), (x, y_range.end)], &BLACK.mix(0.75))
    });
    chart.draw_series(vlines)?;

    let annotations = x_map.iter().map(|(num_elements, i)| {
        let x = (group_width * i) + (group_width / 2);
        Text::new(format!("{}", num_elements), (x, 0), ("sans-serif", 18))
    });
    chart.draw_series(annotations)?;

    let data = results
        .into_iter()
        .map(|x| (x.title.clone(), x))
        .into_group_map();

    for (idx, (title, result)) in data
        .into_iter()
        .sorted_by_key(|(x, _)| x.clone())
        .enumerate()
    {
        let color = Palette99::pick(idx).mix(0.75);
        let x_offset = group_pad + bar_width * idx;
        let series = get_plot_data(result).into_iter().map(|(x, y)| {
            let x = *x_map.get(&x).unwrap();
            let x0 = x * group_width + x_offset;
            Rectangle::new([(x0, y), (x0 + bar_width, 0)], color.filled())
        });
        chart
            .draw_series(series)?
            .label(title)
            .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled()));
    }

    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .background_style(&WHITE.mix(0.8))
        .position(SeriesLabelPosition::UpperLeft)
        .draw()?;

    root.present()?;
    println!("A plot has been saved to: {}", OUT_FILE_NAME);
    Ok(())
}

/// Extracts x & y coordinates from `data`
fn get_plot_data(data: Vec<TestResult>) -> Vec<(usize, u128)> {
    data.into_iter()
        .map(|x| (x.num_elements, x.elapsed.as_millis()))
        .collect::<Vec<_>>()
}
