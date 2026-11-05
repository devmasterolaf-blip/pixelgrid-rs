mod common;

use common::{mask, picture};
use pixelgrid::label::{label_components, label_nonzero, Labeling};
use pixelgrid::{Connectivity, Gray, Image, Rect, Rgb};

/// Renders the label map with one digit per pixel.
fn digits(labeling: &Labeling) -> Vec<String> {
    labeling
        .labels()
        .chunks(labeling.width() as usize)
        .map(|row| row.iter().map(|l| char::from_digit(*l, 36).unwrap()).collect())
        .collect()
}

#[test]
fn separate_blobs_are_numbered_in_scan_order() {
    let img = mask(&["##..#", "##..#", ".....", "..##."]);
    let labeling = label_nonzero(&img, Connectivity::Four);
    assert_eq!(digits(&labeling), ["11002", "11002", "00000", "00330"]);
    assert_eq!(labeling.count(), 3);
    let c = labeling.components();
    assert_eq!((c[0].label, c[0].area, c[0].bounds), (1, 4, Rect::new(0, 0, 2, 2)));
    assert_eq!((c[1].label, c[1].area, c[1].bounds), (2, 2, Rect::new(4, 0, 1, 2)));
    assert_eq!((c[2].label, c[2].area, c[2].bounds), (3, 2, Rect::new(2, 3, 2, 1)));
    assert_eq!(c[0].centroid, (0.5, 0.5));
    assert_eq!(c[1].centroid, (4.0, 0.5));
    assert_eq!(c[2].centroid, (2.5, 3.0));
}

#[test]
fn diagonal_neighbours_join_only_with_eight_connectivity() {
    let img = mask(&["#..", ".#.", "..#"]);
    let four = label_nonzero(&img, Connectivity::Four);
    assert_eq!(digits(&four), ["100", "020", "003"]);
    let eight = label_nonzero(&img, Connectivity::Eight);
    assert_eq!(digits(&eight), ["100", "010", "001"]);
    assert_eq!(eight.components()[0].bounds, Rect::new(0, 0, 3, 3));
    assert_eq!(eight.components()[0].centroid, (1.0, 1.0));
    // The anti-diagonal needs the upper-right neighbour.
    let anti = mask(&["..#", ".#.", "#.."]);
    assert_eq!(label_nonzero(&anti, Connectivity::Eight).count(), 1);
    assert_eq!(label_nonzero(&anti, Connectivity::Four).count(), 3);
}

#[test]
fn u_shape_merges_provisional_labels() {
    // The two arms get different provisional labels and are joined by the bottom row.
    let img = mask(&["#.#", "#.#", "###"]);
    let labeling = label_nonzero(&img, Connectivity::Four);
    assert_eq!(digits(&labeling), ["101", "101", "111"]);
    assert_eq!(labeling.count(), 1);
    assert_eq!(labeling.components()[0].area, 7);
}

#[test]
fn merged_labels_leave_no_gaps_in_the_numbering() {
    // Provisional labels: 1 and 2 for the arms of the first U, 3 for the dot. After the arms
    // merge, the dot must become component 2, not 3.
    let img = mask(&["#.#.#", "###..", "....."]);
    let labeling = label_nonzero(&img, Connectivity::Four);
    assert_eq!(digits(&labeling), ["10102", "11100", "00000"]);
    assert_eq!(labeling.count(), 2);
    assert_eq!(labeling.components()[1].bounds, Rect::new(4, 0, 1, 1));
}

#[test]
fn spiral_with_many_merges() {
    let img = mask(&[
        "#######",
        "......#",
        "#####.#",
        "#...#.#",
        "#.###.#",
        "#.....#",
        "#######",
    ]);
    let labeling = label_nonzero(&img, Connectivity::Four);
    assert_eq!(labeling.count(), 1);
    // 7 + 1 + 6 + 3 + 5 + 2 + 7 foreground pixels.
    assert_eq!(labeling.components()[0].area, 31);
    assert_eq!(labeling.components()[0].bounds, Rect::new(0, 0, 7, 7));
    // The background forms one component too.
    let background = label_components(&img, Connectivity::Four, |p| p.0 == 0);
    assert_eq!(background.count(), 1);
    assert_eq!(background.components()[0].area, 18);
}

#[test]
fn staircase_merges_chain_correctly() {
    // Each row introduces a new provisional label to the left that later joins the rest.
    let img = mask(&["...#", "..##", ".##.", "##.."]);
    let labeling = label_nonzero(&img, Connectivity::Four);
    assert_eq!(labeling.count(), 1);
    assert_eq!(digits(&labeling), ["0001", "0011", "0110", "1100"]);
    let comb = mask(&["#.#.#.#", "#.#.#.#", "#######"]);
    let labeling = label_nonzero(&comb, Connectivity::Four);
    assert_eq!(labeling.count(), 1);
    assert_eq!(labeling.components()[0].area, 15);
}

#[test]
fn checkerboard() {
    let img = mask(&["#.#.", ".#.#", "#.#."]);
    assert_eq!(label_nonzero(&img, Connectivity::Four).count(), 6);
    assert_eq!(label_nonzero(&img, Connectivity::Eight).count(), 1);
}

#[test]
fn custom_predicate_on_colour_images() {
    let red = Rgb::new(200, 0, 0);
    let blue = Rgb::new(0, 0, 200);
    let img = Image::from_vec(3, 2, vec![red, blue, red, red, blue, blue]).unwrap();
    let reds = label_components(&img, Connectivity::Four, |p| p.r > 100);
    assert_eq!(reds.count(), 2);
    assert_eq!(reds.components()[0].area, 2);
    let blues = label_components(&img, Connectivity::Four, |p| p.b > 100);
    assert_eq!(blues.count(), 1);
    assert_eq!(blues.components()[0].bounds, Rect::new(1, 0, 2, 2));
}

#[test]
fn masks_and_lookups() {
    let img = mask(&["#.#", "#.#"]);
    let labeling = label_nonzero(&img, Connectivity::Four);
    assert_eq!(picture(&labeling.mask(1)), ["#..", "#.."]);
    assert_eq!(picture(&labeling.mask(2)), ["..#", "..#"]);
    assert_eq!(picture(&labeling.mask(0)), ["...", "..."]);
    assert_eq!(picture(&labeling.mask(9)), ["...", "..."]);
    assert_eq!(labeling.label_at(2, 1), Some(2));
    assert_eq!(labeling.label_at(1, 1), Some(0));
    assert_eq!(labeling.label_at(3, 0), None);
    assert_eq!(labeling.label_at(0, 2), None);
    assert_eq!((labeling.width(), labeling.height()), (3, 2));
}

#[test]
fn empty_and_blank_images_have_no_components() {
    let empty: Image<Gray> = Image::new(0, 0);
    let labeling = label_nonzero(&empty, Connectivity::Eight);
    assert_eq!((labeling.count(), labeling.labels().len()), (0, 0));
    let blank: Image<Gray> = Image::new(4, 3);
    let labeling = label_nonzero(&blank, Connectivity::Eight);
    assert_eq!(labeling.count(), 0);
    assert!(labeling.labels().iter().all(|l| *l == 0));
    let full: Image<Gray> = Image::filled(4, 3, Gray(1));
    let labeling = label_nonzero(&full, Connectivity::Four);
    assert_eq!(labeling.count(), 1);
    assert_eq!(labeling.components()[0].area, 12);
    assert_eq!(labeling.components()[0].centroid, (1.5, 1.0));
}
