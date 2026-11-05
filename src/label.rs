//! Connected-component labelling.

use crate::image::{Connectivity, Image, Rect};
use crate::pixel::{Gray, Pixel};

/// One connected group of foreground pixels.
#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    /// The label of this component, starting at 1.
    pub label: u32,
    /// Number of pixels.
    pub area: usize,
    /// The smallest rectangle containing every pixel of the component.
    pub bounds: Rect,
    /// Mean pixel position `(x, y)`.
    pub centroid: (f64, f64),
}

/// The result of [`label_components`]: a label for every pixel and a summary per component.
#[derive(Debug, Clone, PartialEq)]
pub struct Labeling {
    width: u32,
    height: u32,
    labels: Vec<u32>,
    components: Vec<Component>,
}

impl Labeling {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Labels in row order: 0 for background, `1..=count` for the components.
    pub fn labels(&self) -> &[u32] {
        &self.labels
    }

    /// The label at `(x, y)`, or `None` outside the image.
    pub fn label_at(&self, x: u32, y: u32) -> Option<u32> {
        if x < self.width && y < self.height {
            Some(self.labels[y as usize * self.width as usize + x as usize])
        } else {
            None
        }
    }

    /// Number of components.
    pub fn count(&self) -> usize {
        self.components.len()
    }

    /// The components ordered by label, which is the order in which a row-by-row scan first
    /// meets them.
    pub fn components(&self) -> &[Component] {
        &self.components
    }

    /// A mask that is 255 on the pixels of component `label` and 0 elsewhere. Label 0 or an
    /// unknown label gives an all-zero mask.
    pub fn mask(&self, label: u32) -> Image<Gray> {
        let data = self.labels.iter().map(|l| Gray(if *l == label && label != 0 { 255 } else { 0 })).collect();
        Image::from_vec(self.width, self.height, data).expect("label buffer matches the image size")
    }
}

/// Follows parent links to the representative of `label`, halving the path on the way.
fn find(parents: &mut [u32], mut label: u32) -> u32 {
    while parents[label as usize] != label {
        let grandparent = parents[parents[label as usize] as usize];
        parents[label as usize] = grandparent;
        label = grandparent;
    }
    label
}

/// Labels the connected groups of pixels for which `is_foreground` returns true.
///
/// Uses the classic two-pass algorithm: a raster scan hands out provisional labels and records
/// in a union-find structure which of them touch; a second pass replaces every provisional
/// label by a final one. Final labels are numbered from 1 in the order a row-by-row scan first
/// meets each component, so the result does not depend on how provisional labels were merged.
pub fn label_components<P: Pixel, F: Fn(P) -> bool>(
    image: &Image<P>,
    connectivity: Connectivity,
    is_foreground: F,
) -> Labeling {
    let (width, height) = (image.width() as usize, image.height() as usize);
    let mut labels = vec![0u32; width * height];
    // parents[0] is the background and never merged with anything.
    let mut parents: Vec<u32> = vec![0];
    // Neighbours a raster scan has already visited.
    let earlier: &[(i64, i64)] = match connectivity {
        Connectivity::Four => &[(-1, 0), (0, -1)],
        Connectivity::Eight => &[(-1, 0), (-1, -1), (0, -1), (1, -1)],
    };
    for y in 0..height {
        for x in 0..width {
            if !is_foreground(image.row(y as u32)[x]) {
                continue;
            }
            let mut current = 0u32;
            for &(dx, dy) in earlier {
                let (nx, ny) = (x as i64 + dx, y as i64 + dy);
                if nx < 0 || ny < 0 || nx >= width as i64 {
                    continue;
                }
                let neighbour = labels[ny as usize * width + nx as usize];
                if neighbour == 0 {
                    continue;
                }
                let root = find(&mut parents, neighbour);
                if current == 0 {
                    current = root;
                } else if root != current {
                    let (low, high) = (current.min(root), current.max(root));
                    parents[high as usize] = low;
                    current = low;
                }
            }
            if current == 0 {
                current = parents.len() as u32;
                parents.push(current);
            }
            labels[y * width + x] = current;
        }
    }

    struct Stats {
        area: usize,
        min: (u32, u32),
        max: (u32, u32),
        sum: (u64, u64),
    }
    let mut final_labels = vec![0u32; parents.len()];
    let mut stats: Vec<Stats> = Vec::new();
    for (index, label) in labels.iter_mut().enumerate() {
        if *label == 0 {
            continue;
        }
        let root = find(&mut parents, *label) as usize;
        let (x, y) = ((index % width) as u32, (index / width) as u32);
        if final_labels[root] == 0 {
            stats.push(Stats { area: 0, min: (x, y), max: (x, y), sum: (0, 0) });
            final_labels[root] = stats.len() as u32;
        }
        *label = final_labels[root];
        let s = &mut stats[*label as usize - 1];
        s.area += 1;
        s.min = (s.min.0.min(x), s.min.1.min(y));
        s.max = (s.max.0.max(x), s.max.1.max(y));
        s.sum = (s.sum.0 + u64::from(x), s.sum.1 + u64::from(y));
    }
    let components = stats
        .iter()
        .enumerate()
        .map(|(i, s)| Component {
            label: i as u32 + 1,
            area: s.area,
            bounds: Rect::new(s.min.0, s.min.1, s.max.0 - s.min.0 + 1, s.max.1 - s.min.1 + 1),
            centroid: (s.sum.0 as f64 / s.area as f64, s.sum.1 as f64 / s.area as f64),
        })
        .collect();
    Labeling { width: image.width(), height: image.height(), labels, components }
}

/// Labels the components of non-zero pixels in a grey image, the usual convention for masks.
pub fn label_nonzero(image: &Image<Gray>, connectivity: Connectivity) -> Labeling {
    label_components(image, connectivity, |p| p.0 != 0)
}
