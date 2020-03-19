use plotters::prelude::*;
use web_sys::HtmlCanvasElement;
use crate::data::*;

// TODO: create an appropriate error type
pub fn draw(data_set : &DataSet, series : Vec<&Series>, el: HtmlCanvasElement) -> Result<(), &'static str> {
    let backend = CanvasBackend::with_canvas_object(el).ok_or("unable to retrieve canvas context")?;

    let root = backend.into_drawing_area();
    root.fill(&BLACK).map_err(|_| "unable to fill")?;

    let days : usize = data_set.dates.len();

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .x_label_area_size(75)
        .y_label_area_size(75)
        .build_ranged(0..days, 0.0..max_value(series.iter().map(|x| x.clone())))
        .map_err(|_| "unable to draw chart")?;

    chart
        .configure_mesh()
        .line_style_1(&RGBColor(50, 50, 50))
        .line_style_2(&RGBColor(50, 50, 50))
        .x_desc("Day")
        .y_desc("Cases")
        .x_labels(15)
        .label_style(("sans-serif", 15).into_font().color(&WHITE))
        .draw()
        .map_err(|_| "unable to draw mesh")?;

    for &s in &series {
        chart.draw_series(
            LineSeries::new(s.points.iter().map(|x| *x).enumerate(), &RED),
        ).unwrap()
        .label(&s.region.country);
    }

    //chart.draw_series(
        //LineSeries::new((0..100).map(|x| (x, 100 - x)), &WHITE),
    //).unwrap();

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
