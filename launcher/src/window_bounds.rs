//! Where the launcher's window was last, kept in `Settings::window`.

/// Where the window was last, kept in `Settings::window` so the next start opens it there.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SavedBounds {
    /// The window filled the screen.
    Maximized,
    /// A windowed rectangle in logical pixels: x, y, width, height.
    Windowed {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
}

impl SavedBounds {
    /// The windowed rectangle, for tests and for a caller that wants the numbers alone.
    pub fn to_gpui_rect(self) -> (f32, f32, f32, f32) {
        match self {
            SavedBounds::Maximized => (0., 0., 1180., 760.),
            SavedBounds::Windowed {
                x,
                y,
                width,
                height,
            } => (x, y, width.max(960.), height.max(600.)),
        }
    }

    /// The GPUI bounds for this saved place.
    pub fn to_gpui(self, cx: &gpui::App) -> gpui::WindowBounds {
        use gpui::{px, size, Bounds, Point, WindowBounds};
        match self {
            SavedBounds::Maximized => WindowBounds::Maximized(Bounds::centered(
                None,
                size(px(1180.), px(760.)),
                cx,
            )),
            SavedBounds::Windowed {
                x,
                y,
                width,
                height,
            } => {
                // A window smaller than the layout needs is widened: the pages assume 960 px.
                let width = width.max(960.);
                let height = height.max(600.);
                WindowBounds::Windowed(Bounds::new(
                    Point::new(px(x), px(y)),
                    size(px(width), px(height)),
                ))
            }
        }
    }

    /// The bounds a window has now, ready to be saved. A full screen window is stored as the
    /// windowed rectangle it would return to, so un-maximising after a restart works.
    pub fn of(bounds: gpui::WindowBounds) -> Option<Self> {
        use gpui::WindowBounds;
        let f = |v: gpui::Pixels| f32::from(v);
        match bounds {
            WindowBounds::Maximized(_) | WindowBounds::Fullscreen(_) => Some(SavedBounds::Maximized),
            WindowBounds::Windowed(b) => {
                let (w, h) = (f(b.size.width), f(b.size.height));
                // A minimised or not-yet-laid-out window reports nothing usable.
                if w < 320. || h < 240. {
                    return None;
                }
                Some(SavedBounds::Windowed {
                    x: f(b.origin.x),
                    y: f(b.origin.y),
                    width: w,
                    height: h,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A window too small to lay the pages out is widened to the minimum, and a saved place is
    /// read back exactly as it was saved.
    #[test]
    fn a_saved_place_round_trips_and_keeps_the_pages_readable() {
        let saved = SavedBounds::Windowed {
            x: 40.,
            y: 30.,
            width: 1400.,
            height: 900.,
        };
        assert_eq!(saved.to_gpui_rect(), (40., 30., 1400., 900.));

        // Smaller than the layout's floor: widened, never shrunk further.
        let tiny = SavedBounds::Windowed {
            x: 0.,
            y: 0.,
            width: 200.,
            height: 100.,
        };
        assert_eq!(tiny.to_gpui_rect(), (0., 0., 960., 600.));
    }

    /// What is saved is what a windowed window has, and a full-screen one comes back maximised.
    #[test]
    fn windowed_bounds_are_saved_as_they_are() {
        use gpui::{point, px, size, Bounds, WindowBounds};
        let bounds = WindowBounds::Windowed(Bounds::new(point(px(12.), px(34.)), size(px(1200.), px(800.))));
        assert_eq!(
            SavedBounds::of(bounds),
            Some(SavedBounds::Windowed {
                x: 12.,
                y: 34.,
                width: 1200.,
                height: 800.,
            })
        );

        // A window that has not been laid out yet (or is minimised) says nothing usable.
        let empty = WindowBounds::Windowed(Bounds::new(point(px(0.), px(0.)), size(px(0.), px(0.))));
        assert_eq!(SavedBounds::of(empty), None);
    }
}
