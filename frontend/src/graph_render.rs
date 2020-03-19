use plotters::prelude::*;
use web_sys::HtmlCanvasElement;
use crate::data::*;
use num_format::{Locale, ToFormattedString, ToFormattedStr};

// TODO: create an appropriate error type
pub fn draw(data_set : &DataSet, series : Vec<&Series>, el: HtmlCanvasElement) -> Result<(), &'static str> {
    let backend = CanvasBackend::with_canvas_object(el).ok_or("unable to retrieve canvas context")?;

    let root = backend.into_drawing_area();
    root.fill(&BLACK).map_err(|_| "unable to fill")?;

    let days : usize = data_set.dates.len();

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .x_label_area_size(75)
        .y_label_area_size(100)
        .build_ranged(0..days, 0.0..max_value(series.iter().map(|x| x.clone()))*1.05)
        .map_err(|_| "unable to draw chart")?;

    chart
        .configure_mesh()
        .line_style_1(&RGBColor(50, 50, 50))
        .line_style_2(&RGBColor(50, 50, 50))
        .x_desc("Day")
        .y_desc("Cases")
        .x_labels(15)
        .x_label_formatter(&|i| data_set.dates.get(*i).map(|d| d.format("%b %d").to_string()).unwrap_or_default())
        .y_label_formatter(&|v| {
            let v_u64 : u64 = *v as u64;
            v_u64.to_formatted_string(&Locale::en)
        })
        .label_style(("sans-serif", 15).into_font().color(&WHITE))
        .draw()
        .map_err(|_| "unable to draw mesh")?;

    for (idx, &s) in series.iter().enumerate() {
        chart.draw_series(
            LineSeries::new(s.points.iter().map(|x| *x).enumerate(), &Palette99::pick(idx)),
        ).unwrap()
        .label(&s.region.country)
        .legend(move |(x, y)| {
            Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], &Palette99::pick(idx))
        });
    }

    Ok(())
}

fn max_value<'a, I: Iterator<Item = &'a Series>>(series : I) -> f64 {
    let mut max = 0f64;
    for s in series {
        for pt in &s.points {
            if *pt > max {
                max = *pt;
            }
        }
    }
    max
}
