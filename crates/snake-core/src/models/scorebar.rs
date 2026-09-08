use std::{
    cmp::Ordering,
    fmt::{self, Display},
    iter::Sum,
    ops::{Add, AddAssign},
};

use crate::{
    GameObject, GameObjectId, Render, RenderItem, Rotation, Texture, TextureColor, Vector2Int,
    ZIndex,
};

#[derive(Debug, Clone)]
pub struct Scorebar {
    id: GameObjectId,
    pub score: Score,
    /// saved scores of completed games
    saved_scores: SavedScores,
    position: Vector2Int,
}

impl Scorebar {
    const SCORES_CAPACITY: usize = 100;
    #[must_use]
    pub fn new(position: Vector2Int) -> Self {
        Self {
            id: GameObjectId::new(),
            score: Score::default(),
            saved_scores: SavedScores::with_capacity(Self::SCORES_CAPACITY),
            position,
        }
    }

    #[must_use]
    pub const fn high_score(&self) -> Score {
        match self.saved_scores.high() {
            Some(high_score) => high_score,
            None => Score::EMPTY,
        }
    }

    #[must_use]
    pub const fn saved_score_total(&self) -> Score {
        self.saved_scores.total()
    }

    pub fn reset(&mut self) {
        self.score.reset();
    }

    /// resets score and adds to total
    pub fn save_and_reset(&mut self) {
        self.save_score();
        self.score.reset();
    }

    fn save_score(&mut self) {
        self.saved_scores.push(self.score);
    }
}

impl GameObject for Scorebar {
    fn id(&self) -> GameObjectId {
        self.id
    }

    fn position(&self) -> Vector2Int {
        self.position
    }

    fn rotation(&self) -> Rotation {
        Rotation::RIGHT
    }
}

impl Render for Scorebar {
    fn visit_render_items(&self, visit: &mut dyn FnMut(RenderItem)) {
        let score = self.score;
        let high_score = self.high_score();
        let saved_score_total = self.saved_score_total();

        let (score_color, high_score_color) = match score.cmp(&high_score) {
            Ordering::Greater => (TextureColor::Gold, TextureColor::White),
            Ordering::Equal => (TextureColor::Gold, TextureColor::Gold),
            Ordering::Less => (TextureColor::White, TextureColor::Gold),
        };
        let rows = [
            ("Score", score, score_color),
            ("High", high_score, high_score_color),
            ("Total", saved_score_total, TextureColor::White),
        ];
        for (row, (label, value, color)) in (0..).zip(rows) {
            let content = format!("{label}: {value}");
            for (column, character) in (0..).zip(content.chars()) {
                visit(RenderItem::screen_glyph(
                    Vector2Int::new(column, row),
                    Texture::new(character, color),
                    ZIndex::new(1),
                ));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Score(u32);
impl Score {
    pub const EMPTY: Self = Score(0);
    pub const UNIT: Self = Score(100);

    #[must_use]
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// adds a unit of score: `Score::UNIT`
    pub fn add_unit(&mut self) {
        self.0 += Self::UNIT.0;
    }

    pub(in crate::models::scorebar) fn reset(&mut self) {
        self.0 = Self::default().0;
    }
}
impl Default for Score {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl AddAssign for Score {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_add(rhs.0);
    }
}
impl Add for Score {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_add(rhs.0))
    }
}
impl Sum for Score {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut sum = Self::EMPTY;
        for score in iter {
            sum += score;
        }
        sum
    }
}
impl Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_score(*self, f)
    }
}
pub(in crate::models::scorebar) fn format_score(
    score: Score,
    output: &mut impl fmt::Write,
) -> fmt::Result {
    match score.0 {
        value @ 0..1_000 => write!(output, "{value}"),
        value @ 1_000..=u32::MAX => {
            let thousands = value / 1_000;
            let hundreds = value % 1_000 / 100;
            write!(output, "{thousands}.{hundreds}k")
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SavedScores {
    values: Vec<Score>,
    total: Score,
    high: Option<Score>,
}

impl SavedScores {
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
            total: Score::EMPTY,
            high: None,
        }
    }

    pub fn push(&mut self, score: Score) {
        self.values.push(score);
        self.total += score;
        self.high = Some(self.high.map_or(score, |high| high.max(score)));
    }

    #[must_use]
    pub const fn total(&self) -> Score {
        self.total
    }

    #[must_use]
    pub const fn high(&self) -> Option<Score> {
        self.high
    }

    #[must_use]
    pub fn as_slice(&self) -> &[Score] {
        &self.values
    }
}
#[cfg(test)]
mod tests {
    use crate::models::scorebar::Score;

    #[test]
    fn display_formats_below_1000() {
        assert_eq!(Score::EMPTY.to_string(), "0");
        assert_eq!(Score(486).to_string(), "486");
    }

    #[test]
    fn display_formats_above_1000() {
        assert_eq!(Score(1863).to_string(), "1.8k");
        assert_eq!(Score(1000).to_string(), "1.0k");
        assert_eq!(Score(4_153_800).to_string(), "4153.8k");
        assert_eq!(Score(19000).to_string(), "19.0k");
    }
}
