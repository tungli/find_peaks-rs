use core::ops::Range;
use std::borrow::Cow;

/// Struct containing the information of a found peak.
///
/// Some values can be `None`s -- you have to specify at least one of the corresponding bounds in
/// `PeakFinder`. If you don't, `find_peaks` skipps their calculation.
#[derive(Debug, PartialEq, Clone)]
pub struct Peak<T> {
    /// range indices the peak spans
    pub position: Range<usize>,
    /// absolute value of difference to the nearest neighbour to the left
    pub left_diff: T,
    /// absolute value of difference to the nearest neighbour to the right
    pub right_diff: T,
    pub height: Option<T>,
    pub prominence: Option<T>,
}

impl<T> Peak<T> {
    fn new(position: Range<usize>, left_diff: T, right_diff: T) -> Self {
        Self {
            position,
            left_diff,
            right_diff,
            height: None,
            prominence: None,
        }
    }
    fn add_height(&mut self, h: T) {
        self.height = Some(h);
    }
    fn add_prominence(&mut self, p: T) {
        self.prominence = Some(p);
    }

    /// Get the middle index of a peak (plateau). For an even plateau size the function rounds down.
    pub fn middle_position(&self) -> usize {
        (self.position.start + self.position.end) / 2
    }
}

#[derive(Debug, Clone)]
struct Limits<T> {
    pub lower: Option<T>,
    pub upper: Option<T>,
}

impl<T> Limits<T>
where
    T: PartialOrd,
{
    pub fn empty() -> Self {
        Self {
            lower: None,
            upper: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lower.is_none() && self.upper.is_none()
    }

    pub fn is_inside(&self, v: &T) -> bool {
        (self.lower.is_none() || v.ge(self.lower.as_ref().unwrap()))
            && (self.upper.is_none() || v.le(self.upper.as_ref().unwrap()))
    }
}

/// Setup for the peak filtering.
///
/// Change the settings by using the methods for specifing the lower and upper bounds.
#[derive(Clone)]
pub struct PeakFinder<'a, T, S>
    where [S]: ToOwned
{
    y_data: &'a [T],
    x_data: Cow<'a, [S]>,
    height: Limits<T>,
    prominence: Limits<T>,
    difference: Limits<T>,
    plateau_size: Limits<usize>,
    distance: Limits<S>,
    zero: Option<T>,
}

impl<'a, T> PeakFinder<'a, T, usize>
where
    T: Clone + std::ops::Sub<Output = T> + PartialOrd
{
    /// Initialize with a data slice.
    pub fn new(y_data: &'a [T]) -> Self {
        let x: Vec<usize> = (0..y_data.len()).collect();
        if y_data.is_empty() {
            Self {
                y_data,
                x_data: Cow::from(x),
                height: Limits::empty(),
                prominence: Limits::empty(),
                difference: Limits::empty(),
                plateau_size: Limits::empty(),
                distance: Limits::empty(),
                zero: None,
            }
        } else {
            let zero = Some(y_data[0].clone() - y_data[0].clone());
            Self {
                y_data,
                x_data: Cow::from(x),
                height: Limits::empty(),
                prominence: Limits::empty(),
                difference: Limits {
                    lower: zero.clone(),
                    upper: None,
                },
                plateau_size: Limits::empty(),
                distance: Limits::empty(),
                zero,
            }
        }
    }
}

impl<'a, T, S> PeakFinder<'a, T, S>
where
    T: Clone + std::ops::Sub<Output = T> + PartialOrd,
    S: Clone + std::ops::Sub<Output = S> + PartialOrd,
    [S]: ToOwned,
{
    pub fn new_with_x(y_data: &'a [T], x_data: &'a [S]) -> Self {
        if y_data.is_empty() {
            Self {
                y_data,
                x_data: Cow::from(x_data),
                height: Limits::empty(),
                prominence: Limits::empty(),
                difference: Limits::empty(),
                plateau_size: Limits::empty(),
                distance: Limits::empty(),
                zero: None,
            }
        } else {
            let zero = Some(y_data[0].clone() - y_data[0].clone());
            Self {
                y_data,
                x_data: Cow::from(x_data),
                height: Limits::empty(),
                prominence: Limits::empty(),
                difference: Limits {
                    lower: zero.clone(),
                    upper: None,
                },
                plateau_size: Limits::empty(),
                distance: Limits::empty(),
                zero,
            }
        }
    }

    fn get_local_maxima<'b>(&'b self) -> impl Iterator<Item = Peak<T>> + 'b {
        let zero = self.zero.clone().unwrap();

        let mut it = self.y_data.iter().cloned().enumerate();
        let (_i, zeroth) = it.next().unwrap();
        let (_i, first) = it.next().unwrap();

        let mut back_diff = first.clone() - zeroth;
        let mut prev = first;

        let limit = &self.difference;

        let mut start: Option<usize> = None;

        it.filter_map(move |(i, y)| {
            let ahead_diff = prev.clone() - y.clone(); // positive for downward slope
            let ahead_inside = limit.is_inside(&ahead_diff);
            let back_inside = limit.is_inside(&back_diff);

            let res = if back_inside && ahead_diff == zero {
                if start.is_none() {
                    start = Some(i - 1);
                }
                None
            } else {
                let r = if ahead_inside && back_inside {
                    Some(Peak::new(
                        start.unwrap_or(i - 1)..i,
                        back_diff.clone(),
                        ahead_diff.clone(),
                    ))
                } else {
                    None
                };

                start = None;
                back_diff = zero.clone() - ahead_diff;

                r
            };
            prev = y.clone();

            res
        })
    }

    fn filter_plateau<'b, I>(&'b self, peaks: I) -> impl Iterator<Item = Peak<T>> + 'b
    where
        I: Iterator<Item = Peak<T>> + 'b,
    {
        let limit = &self.plateau_size;
        let empty = limit.is_empty();

        peaks.filter_map(move |p| {
            if empty {
                // do nothing
                Some(p)
            } else {
                if limit.is_inside(&p.position.len()) {
                    Some(p)
                } else {
                    None
                }
            }
        })
    }

    fn filter_height<'b, I>(&'b self, peaks: I) -> impl Iterator<Item = Peak<T>> + 'b
    where
        I: Iterator<Item = Peak<T>> + 'b,
    {
        let limit = &self.height;
        let empty = limit.is_empty();

        peaks.filter_map(move |mut p| {
            if empty {
                // do nothing
                Some(p)
            } else {
                let y = self.y_data[p.position.start].clone();

                if limit.is_inside(&y) {
                    p.add_height(y);
                    Some(p)
                } else {
                    None
                }
            }
        })
    }

    fn filter_prominence<'b, I>(&'b self, peaks: I) -> impl Iterator<Item = Peak<T>> + 'b
    where
        I: Iterator<Item = Peak<T>> + 'b,
    {
        let limit = &self.prominence;
        let empty = limit.is_empty();

        peaks.filter_map(move |mut p| {
            if empty {
                // do nothing
                Some(p)
            } else {
                let prom = self.calc_prominence(&p);

                if limit.is_inside(&prom) {
                    p.add_prominence(prom);
                    Some(p)
                } else {
                    None
                }
            }
        })
    }

    fn filter_distance(&self, mut peaks: Vec<Peak<T>>) -> Vec<Peak<T>>
    {   
        peaks.sort_unstable_by(|a, b| b.height.partial_cmp(&a.height).unwrap_or(std::cmp::Ordering::Equal));

        let limit = &self.distance;
        if limit.is_empty() {
            return peaks;
        }   

        let mut filtered = Vec::with_capacity(peaks.len());
        filtered.push(peaks[0].clone());

        let x_data = &self.x_data;
        let mut x_prev = x_data[peaks[0].middle_position()].clone();
        for i in 1..peaks.len() {
            let x = x_data[peaks[i].middle_position()].clone();
            let dist = if x > x_prev {
                x.clone() - x_prev.clone()
            } else {
                x_prev.clone() - x.clone()
            };  
            if limit.is_inside(&dist) {
                filtered.push(peaks[i].clone());
                x_prev = x.clone();
            }   
        }   

        filtered.shrink_to_fit();
        filtered
    }  

    fn calc_prominence(&self, p: &Peak<T>) -> T {
        let i_left = p.position.start;
        let i_right = p.position.end - 1;

        let data = &self.y_data;

        //debug_assert_eq!(data[i_right], data[i_left]);

        let from_peak_right = data.iter().skip(i_right + 1);
        let from_peak_left = data.iter().rev().skip(data.len() - i_left);

        let left_valley_y = from_peak_left
            .take_while(|&x| x <= &data[i_left])
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let right_valley_y = from_peak_right
            .take_while(|&x| x <= &data[i_left])
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let peak_height = data[i_left].clone();
        match (left_valley_y, right_valley_y) {
            (None, None) => self.zero.clone().unwrap(),
            (Some(v), None) => peak_height - v.clone(),
            (None, Some(v)) => peak_height - v.clone(),
            (Some(v1), Some(v2)) => peak_height - (if v1.ge(&v2) { v1 } else { v2 }).clone(),
        }
        .clone()
    }

    /// Outputs a vector of `Peak<_>` structures containing peaks that matched the criteria
    /// specified in `PeakFinder<_>`.
    ///
    /// Output will **not** contain some properties (for example, height, prominence) unless you
    /// specified at least on of the corresponding bounds in `PeakFinder<_>` -- the calculation of
    /// the property is skipped.
    ///
    /// Peaks are sorted by their height.
    ///
    /// # Examples
    ///
    /// ```
    /// use find_peaks::PeakFinder;
    /// let y = [1., 2., 3., 0., 5., 0.];
    ///
    /// let ps = PeakFinder::new(&y)
    ///            .with_min_height(0.)
    ///            .with_min_prominence(1.)
    ///            .find_peaks();
    ///
    /// assert_eq!(
    ///    ps.iter().map(|x| x.middle_position()).collect::<Vec<_>>(),
    ///    vec![4, 2]
    /// );
    /// ```
    pub fn find_peaks(&self) -> Vec<Peak<T>> {
        if [0, 1].contains(&self.y_data.len()) {
            return Vec::new();
        }

        let it = self
            .filter_prominence(self.filter_height(self.filter_plateau(self.get_local_maxima())));

        let peaks = it.collect();
        self.filter_distance(peaks)
    }

    pub fn with_min_height(&mut self, h: T) -> &mut Self {
        self.height.lower = Some(h);
        self
    }

    pub fn with_max_height(&mut self, h: T) -> &mut Self {
        self.height.upper = Some(h);
        self
    }

    pub fn with_min_prominence(&mut self, prominence: T) -> &mut Self {
        let zero = prominence.clone() - prominence.clone();
        assert!(zero.le(&prominence), "Prominence must be positive!");

        self.prominence.lower = Some(prominence);
        self
    }

    pub fn with_max_prominence(&mut self, prominence: T) -> &mut Self {
        let zero = prominence.clone() - prominence.clone();
        assert!(zero.le(&prominence), "Prominence must be positive!");

        self.prominence.upper = Some(prominence);
        self
    }

    pub fn with_min_difference(&mut self, difference: T) -> &mut Self {
        let zero = difference.clone() - difference.clone();
        assert!(zero.le(&difference), "Difference must be positive!");

        self.difference.lower = Some(difference);
        self
    }

    pub fn with_max_difference(&mut self, difference: T) -> &mut Self {
        let zero = difference.clone() - difference.clone();
        assert!(zero.le(&difference), "Difference must be positive!");

        self.difference.upper = Some(difference);
        self
    }

    pub fn with_min_plateau_size(&mut self, size: usize) -> &mut Self {
        self.plateau_size.lower = Some(size);
        self
    }

    pub fn with_max_plateau_size(&mut self, size: usize) -> &mut Self {
        self.plateau_size.upper = Some(size);
        self
    }

    pub fn with_min_distance(&mut self, distance: S) -> &mut Self {
        let zero = distance.clone() - distance.clone();
        assert!(zero.le(&distance), "Distance must be positive!");

        self.distance.lower = Some(distance);
        self
    }

    pub fn with_max_distance(&mut self, distance: S) -> &mut Self {
        let zero = distance.clone() - distance.clone();
        assert!(zero.le(&distance), "Distance must be positive!");

        self.distance.upper = Some(distance);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{Peak, PeakFinder};

    #[test]
    fn findpeaks() {
        let y = [1., 2., 3., 0., 5., 0.];
        let mut fp = PeakFinder::new(&y);
        fp.with_min_height(0.);
        let ps = fp.find_peaks();
        assert_eq!(
            ps,
            vec![
                Peak {
                    position: 4..5,
                    left_diff: 5.,
                    right_diff: 5.,
                    height: Some(5.),
                    prominence: None
                },
                Peak {
                    position: 2..3,
                    left_diff: 1.,
                    right_diff: 3.,
                    height: Some(3.),
                    prominence: None
                },
            ]
        );
    }

    #[test]
    fn proms() {
        let y = [1., 2., 3., 0., 5., 0.];
        let mut fp = PeakFinder::new(&y);
        fp.with_min_height(0.);
        fp.with_min_prominence(1.);
        let ps = fp.find_peaks();
        assert_eq!(
            ps,
            vec![
                Peak {
                    position: 4..5,
                    left_diff: 5.,
                    right_diff: 5.,
                    height: Some(5.),
                    prominence: Some(5.)
                },
                Peak {
                    position: 2..3,
                    left_diff: 1.,
                    right_diff: 3.,
                    height: Some(3.),
                    prominence: Some(2.)
                }
            ]
        );
    }

    #[test]
    fn plateaus() {
        let y = [1., 2., 3., 3., 3., 0., 5., 5., 0.];
        let mut fp = PeakFinder::new(&y);
        fp.with_min_height(0.);
        fp.with_min_prominence(0.);

        let ps = fp.find_peaks();

        assert_eq!(
            ps,
            vec![
                Peak {
                    position: 6..8,
                    left_diff: 5.,
                    right_diff: 5.,
                    height: Some(5.),
                    prominence: Some(5.)
                },
                Peak {
                    position: 2..5,
                    left_diff: 1.,
                    right_diff: 3.,
                    height: Some(3.),
                    prominence: Some(2.)
                }
            ]
        );

        fp.with_min_plateau_size(3);
        let ps = fp.find_peaks();

        assert_eq!(
            ps,
            vec![Peak {
                position: 2..5,
                left_diff: 1.,
                right_diff: 3.,
                height: Some(3.),
                prominence: Some(2.)
            }]
        );
    }

    #[test]
    fn plateau_with_diff() {
        let y = [1., 2., 3., 3., 3., 0., 5., 5., 0.];
        let mut fp = PeakFinder::new(&y);
        fp.with_min_prominence(0.);
        fp.with_min_height(0.);

        fp.with_min_difference(4.);
        let ps = fp.find_peaks();

        assert_eq!(
            ps,
            vec![Peak {
                position: 6..8,
                left_diff: 5.,
                right_diff: 5.,
                height: Some(5.),
                prominence: Some(5.)
            }]
        );
    }

    #[test]
    fn with_x() {
        let y = [1., 2., 3., 0., 5., 0.];
        let x: Vec<usize> = (1..(y.len() + 1)).collect();
        let ps = PeakFinder::new_with_x(&y, &x)
            .with_min_height(0.)
            .with_min_distance(2)
            .find_peaks();
        assert_eq!(
            ps,
            vec![
                Peak {
                    position: 4..5,
                    left_diff: 5.,
                    right_diff: 5.,
                    height: Some(5.),
                    prominence: None
                },
                Peak {
                    position: 2..3,
                    left_diff: 1.,
                    right_diff: 3.,
                    height: Some(3.),
                    prominence: None
                }
            ]
        );
    }

    #[test]
    fn empty_data() {
        let y: Vec<u8> = vec![];
        let ps = PeakFinder::new(&y).with_min_prominence(1).find_peaks();

        assert_eq!(ps, Vec::new())
    }

    #[test]
    fn single_point() {
        let y: Vec<u32> = vec![1];
        let ps = PeakFinder::new(&y).find_peaks();

        assert_eq!(ps, vec![])
    }

    #[test]
    fn two_points() {
        let y: Vec<u32> = vec![2, 2];
        let ps = PeakFinder::new(&y).find_peaks();

        assert_eq!(ps, vec![])
    }
}
