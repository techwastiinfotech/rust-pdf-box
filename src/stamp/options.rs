use std::ops::RangeInclusive;

/// Page range specification for stamping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageRange {
    All,
    Single(usize),
    Range(RangeInclusive<usize>),
    Odd,
    Even,
    Pages(Vec<usize>),
}

impl PageRange {
    pub fn matches(&self, page_index_0_based: usize, _total_pages: usize) -> bool {
        let page_1_based = page_index_0_based + 1;
        match self {
            Self::All => true,
            Self::Single(p) => *p == page_1_based,
            Self::Range(r) => r.contains(&page_1_based),
            Self::Odd => page_1_based % 2 != 0,
            Self::Even => page_1_based % 2 == 0,
            Self::Pages(list) => list.contains(&page_1_based),
        }
    }
}

/// Stamping layer: whether the stamp sits under or over existing page content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StampLayer {
    UnderContent,
    OverContent,
}

/// Position placement of a stamp on the page.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StampPosition {
    Center,
    TopLeft { margin_x: f64, margin_y: f64 },
    TopCenter { margin_y: f64 },
    TopRight { margin_x: f64, margin_y: f64 },
    BottomLeft { margin_x: f64, margin_y: f64 },
    BottomCenter { margin_y: f64 },
    BottomRight { margin_x: f64, margin_y: f64 },
    Custom { x: f64, y: f64 },
}
