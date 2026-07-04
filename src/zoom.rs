pub struct ZoomCalculator;

impl ZoomCalculator {
    pub fn calculate_fit_scale(
        image_width: u32,
        image_height: u32,
        window_width: u32,
        window_height: u32,
    ) -> f32 {
        if image_width == 0 || image_height == 0 || window_width == 0 || window_height == 0 {
            return 1.0;
        }

        let width_ratio = window_width as f32 / image_width as f32;
        let height_ratio = window_height as f32 / image_height as f32;

        width_ratio.min(height_ratio)
    }

    pub fn calculate_scaled_dimensions(
        original_width: u32,
        original_height: u32,
        zoom_level: f32,
    ) -> (u32, u32) {
        let width = (original_width as f32 * zoom_level).round() as u32;
        let height = (original_height as f32 * zoom_level).round() as u32;
        (width.max(1), height.max(1))
    }

    pub fn zoom_in(current: f32) -> f32 {
        let next = (current * 4.0).round() / 4.0 + 0.25;
        next.min(8.0)
    }

    pub fn zoom_out(current: f32) -> f32 {
        let next = (current * 4.0).round() / 4.0 - 0.25;
        next.max(0.25)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fit_scale() {
        assert_eq!(
            ZoomCalculator::calculate_fit_scale(1000, 500, 800, 600),
            0.8
        );
        assert_eq!(
            ZoomCalculator::calculate_fit_scale(500, 1000, 800, 600),
            0.6
        );
    }

    #[test]
    fn test_zoom_in() {
        assert_eq!(ZoomCalculator::zoom_in(1.0), 1.25);
        assert_eq!(ZoomCalculator::zoom_in(1.5), 1.75);
        assert_eq!(ZoomCalculator::zoom_in(7.9), 8.0);
    }

    #[test]
    fn test_zoom_out() {
        assert_eq!(ZoomCalculator::zoom_out(1.0), 0.75);
        assert_eq!(ZoomCalculator::zoom_out(0.5), 0.25);
        assert_eq!(ZoomCalculator::zoom_out(0.25), 0.25);
    }

    #[test]
    fn test_calculate_scaled_dimensions() {
        let (width, height) = ZoomCalculator::calculate_scaled_dimensions(1000, 800, 1.0);
        assert_eq!(width, 1000);
        assert_eq!(height, 800);

        let (width, height) = ZoomCalculator::calculate_scaled_dimensions(1000, 800, 0.5);
        assert_eq!(width, 500);
        assert_eq!(height, 400);

        let (width, height) = ZoomCalculator::calculate_scaled_dimensions(1000, 800, 2.0);
        assert_eq!(width, 2000);
        assert_eq!(height, 1600);
    }

    #[test]
    fn test_fit_scale_edge_cases() {
        // Zero dimensions should return 1.0
        assert_eq!(ZoomCalculator::calculate_fit_scale(0, 0, 800, 600), 1.0);
        assert_eq!(ZoomCalculator::calculate_fit_scale(1000, 800, 0, 0), 1.0);

        // Very large image should scale down
        let scale = ZoomCalculator::calculate_fit_scale(10000, 10000, 800, 600);
        assert!(scale < 0.1);

        // Very small image in large window
        let scale = ZoomCalculator::calculate_fit_scale(100, 100, 2000, 2000);
        assert!(scale > 1.0);
    }

    #[test]
    fn test_zoom_max_and_min_bounds() {
        // Test max zoom (8.0)
        assert_eq!(ZoomCalculator::zoom_in(8.0), 8.0);
        assert_eq!(ZoomCalculator::zoom_in(7.8), 8.0);

        // Test min zoom (0.25)
        assert_eq!(ZoomCalculator::zoom_out(0.25), 0.25);
        assert_eq!(ZoomCalculator::zoom_out(0.3), 0.25);
    }

    #[test]
    fn test_zoom_sequence() {
        let mut level = 1.0;
        level = ZoomCalculator::zoom_in(level);
        assert_eq!(level, 1.25);
        level = ZoomCalculator::zoom_in(level);
        assert_eq!(level, 1.5);
        level = ZoomCalculator::zoom_out(level);
        assert_eq!(level, 1.25);
        level = ZoomCalculator::zoom_out(level);
        assert_eq!(level, 1.0);
    }

    #[test]
    fn test_scaled_dimensions_minimum() {
        // Even with 0 scale, should return at least 1x1
        let (width, height) = ZoomCalculator::calculate_scaled_dimensions(1000, 800, 0.0);
        assert_eq!(width, 1);
        assert_eq!(height, 1);
    }
}
