use plotters::prelude::*;
use web_sys::HtmlCanvasElement;

// TODO: create an appropriate error type
pub fn draw(el: HtmlCanvasElement) -> Result<(), &'static str> {
    let backend = CanvasBackend::with_canvas_object(el).ok_or("unable to retrieve canvas context")?;

    let root = backend.into_drawing_area();
    root.fill(&BLACK).map_err(|_| "unable to fill")?;

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .x_label_area_size(10)
        .y_label_area_size(10)
        .build_ranged(-2.1..0.6, -1.2..1.2)
        .map_err(|_| "unable to draw chart")?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()
        .map_err(|_| "unnable to draw mesh")?;

    let mut chart = ChartBuilder::on(&root)
        .build_ranged(0..100, 0..100)
        .unwrap();

    chart.draw_series(
        LineSeries::new((0..100).map(|x| (x, 100 - x)), &WHITE),
    ).unwrap();

    Ok(())
}
