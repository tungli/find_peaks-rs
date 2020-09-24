use find_peaks::PeakFinder;

use pyo3::prelude::*;

fn main() -> Result<(), ()> {
    Python::with_gil(|py| {
        main_(py).map_err(|e| {
            // We can't display Python exceptions via std::fmt::Display,
            // so print the error here manually.
            e.print_and_set_sys_last_vars(py);
        })
    })
}

fn main_(py: Python) -> PyResult<()> {
    let misc = PyModule::import(py, "scipy.misc")?;
    let data: Vec<f64> = misc.call0("electrocardiogram")?.extract()?;

    let mut fp = PeakFinder::new(&data);
    fp.with_min_prominence(1000. / 400.);
    let peaks = fp.find_peaks();

    let x: Vec<usize> = peaks.iter().map(|x| x.middle_position()).collect();
    let y: Vec<f64> = x.iter().map(|x| data[*x]).collect();

    let plt = PyModule::import(py, "matplotlib.pyplot")?;
    plt.call1("plot", (data,))?;
    plt.call1("plot", (x, y, "o"))?;
    plt.call0("show")?;

    Ok(())
}
