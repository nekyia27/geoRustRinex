//! Meteo observations plotting
use rinex::meteo::*;
use super::{
    Context, Plot2d,
};
use plotters::{
    prelude::*,
    coord::Shift,
    chart::ChartState,
};
use std::collections::HashMap;

/*
 * Builds a plot context for Observation RINEX specificly
 */
pub fn build_context<'a> (dim: (u32, u32), record: &Record) -> Context<'a> {
    let mut e0: i64 = 0;
    let mut t_axis: Vec<f64> = Vec::with_capacity(16384);
    let mut plots: HashMap<String,
        DrawingArea<BitMapBackend, Shift>>
            = HashMap::with_capacity(4);
    let mut y_ranges: HashMap<String, (f64,f64)> = HashMap::new();
    let mut charts: HashMap<String, ChartState<Plot2d>> = HashMap::new();
    //  => 1 plot per physics (ie., Observable)
    for (index, (e, observables)) in record.iter().enumerate() {
        if index == 0 {
            // store first epoch timestamp
            // to scale x_axis proplery (avoids fuzzy rendering)
            e0 = e.date.timestamp();
        }
        let t = e.date.timestamp() - e0;
        t_axis.push(t as f64);
        for (observable, data) in observables {
            if plots.get(&observable.to_string()).is_none() {
                let title = match observable {
                    Observable::Pressure => "pressure.png",
                    Observable::Temperature => "temperature.png",
                    Observable::HumidityRate => "moisture.png",
                    Observable::ZenithWetDelay => "zenith-wet.png",
                    Observable::ZenithDryDelay => "zenith-dry.png",
                    Observable::ZenithTotalDelay => "zenith-total.png",
                    Observable::WindAzimuth => "wind-azim.png",
                    Observable::WindSpeed => "wind-speed.png",
                    Observable::RainIncrement => "rain-increment.png",
                    Observable::HailIndicator=> "hail.png",
                };
                let plot = Context::build_plot(title, dim);
                plots.insert(observable.to_string(), plot);
                y_ranges.insert(observable.to_string(), (*data, *data));
            } else {
                if let Some((min,max)) = y_ranges.get_mut(&observable.to_string()) {
                    if data < min {
                        *min = *data;
                    }
                    if data > max {
                        *max = *data;
                    }
                } else {
                    y_ranges.insert(observable.to_string(), (*data, *data));
                }
            }
        }
    }
    // Add 1 chart onto each plot
    for (id, plot) in plots.iter() {
        // scale this chart nicely
        let range = y_ranges.get(id)
            .unwrap();
        let chart = Context::build_chart(id, t_axis.clone(), *range, plot);
        charts.insert(id.to_string(), chart);
    }
    Context {
        plots,
        charts,
        colors: HashMap::new(), // not needed since we have 1 observable per plot
        t_axis,
    }
}


pub fn plot(ctx: &mut Context, record: &Record) {
    let mut t0 : i64 = 0;
    let mut datasets: HashMap<String, Vec<(f64, f64)>> = HashMap::new();
    for (index, (epoch, observations)) in record.iter().enumerate() {
        if index == 0 {
            t0 = epoch.date.timestamp();
        }
        let t = epoch.date.timestamp();
        for (observable, observation) in observations {
            if let Some(data) = datasets.get_mut(&observable.to_string()) {
                data.push(((t-t0) as f64, *observation));
            } else {
                datasets.insert(observable.to_string(),
                    vec![((t-t0) as f64, *observation)]);
            }
        }
    }

    for (observable, data) in datasets {
        let mut chart = ctx.charts
            .get(&observable)
            .expect(&format!("faulty context, expecting a chart dedicated to \"{}\" observable", observable))
            .clone()
            .restore(ctx.plots.get(&observable.to_string()).unwrap());
        chart
            .draw_series(LineSeries::new(
                data.iter()
                    .map(|(x, y)| (*x, *y)),
                    &BLACK
                ))
            .expect(&format!("failed to draw {} chart", observable))
            .label(observable)
            .legend(|(x, y)| {
                //let color = ctx.colors.get(&vehicule.to_string()).unwrap();
                PathElement::new(vec![(x, y), (x + 20, y)], BLACK)
            });
        chart
            .draw_series(data.iter()
                .map(|point| Cross::new(*point, 4, BLACK.filled())))
                .unwrap();
        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .background_style(WHITE.filled())
            .draw()
            .expect("failed to draw labels on chart");
    }
} 