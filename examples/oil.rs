use find_peaks::PeakFinder;

use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyTuple;
use std::collections::HashSet;

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
    let data = vec![
        78.34, 79.12, 80.12, 80.36, 82.21, 81.43, 81.07, 83.87, 84.9, 84.26, 86.37, 86.51, 88.62,
        87.81, 85.62, 87.98, 85.73, 86.99, 88.45, 88.55, 89.74, 89.67, 89.46, 89.08, 91.23, 93.06,
        92.65, 90.98, 91.66, 91.13, 95.97, 95.76, 93.25, 92.5, 90.32, 91.62, 94.52, 93.53, 94.97,
        97.45, 98.6, 98.5, 107.29, 115.59, 113.2, 127.9, 123.42, 129.9, 113.62, 110.93, 109.72,
        104.08, 100.4, 99.38, 104.31, 107.33, 114.25, 111.93, 119.18, 114.03, 113.72, 107.35,
        109.18, 110.38, 104.68, 103.47, 109.53, 105.46, 101.5, 101.36, 101.03, 99.0, 105.33,
        108.14, 113.0, 112.73, 108.17, 107.54, 108.19, 104.79, 102.42, 105.46, 104.56, 107.0,
        106.74,
    ];

    let prom = 1.;
    for dist in [1, 6, 11, 15, 20, 30].iter() {
        let mut fp = PeakFinder::new(&data);
        fp.with_min_prominence(prom).with_min_distance(*dist);
        let peaks = fp.find_peaks();

        let mut pos = peaks
            .iter()
            .map(|x| x.middle_position())
            .collect::<Vec<_>>();
        pos.sort_unstable();
        //println!("{}   {:?}", dist, pos);

        let x: Vec<usize> = peaks.iter().map(|x| x.middle_position()).collect();
        // let y: Vec<f64> = x.iter().map(|x| data[*x]).collect();

        let signal = PyModule::import(py, "scipy.signal")?;
        let kwargs_vec: Vec<(&str, PyObject)> = vec![
            ("prominence", prom.to_object(py)),
            ("distance", (*dist).to_object(py)),
        ];
        let kwargs = kwargs_vec.into_py_dict(py);
        let scipy_peaks = signal.call("find_peaks", (data.clone(),), Some(kwargs))?;

        let scipy_tuple: &PyTuple = scipy_peaks.downcast()?;
        let scipy_x: Vec<usize> = scipy_tuple.get_item(0).extract()?;
        println!("dist {}\nscipy: {:?}\nfp_rs: {:?}\n", dist, scipy_x, x);

        let scipy_x_set: HashSet<usize> = scipy_x.iter().cloned().collect();
        let x_set: HashSet<usize> = x.iter().cloned().collect();
        assert_eq!(scipy_x_set, x_set);
    }

    // let plt = PyModule::import(py, "matplotlib.pyplot")?;
    // plt.call1("plot", (data,))?;
    // plt.call1("plot", (x, y, "o"))?;
    // plt.call0("show")?;

    Ok(())
}
