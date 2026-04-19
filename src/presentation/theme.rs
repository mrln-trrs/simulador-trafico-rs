use egui::Color32;

/// Definición de la paleta de colores Fluent Design
#[derive(Clone, Copy, Debug)]
pub struct FluentTheme {
    /// Colores base
    pub primary: Color32,
    pub secondary: Color32,
    pub background: Color32,
    pub surface: Color32,
    pub surface_variant: Color32,
    pub outline: Color32,
    pub outline_variant: Color32,
    
    /// Estados
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub info: Color32,
    
    /// Texto
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_tertiary: Color32,
    pub text_disabled: Color32,
    
    /// Interactivos
    pub interactive_primary: Color32,
    pub interactive_hover: Color32,
    pub interactive_pressed: Color32,
    pub interactive_disabled: Color32,
}

impl FluentTheme {
    /// Tema oscuro (por defecto en Fluent)
    pub fn dark() -> Self {
        Self {
            primary: Color32::from_rgb(0, 120, 215),      // Azul Fluent
            secondary: Color32::from_rgb(160, 160, 160),  // Gris claro
            background: Color32::from_rgb(32, 32, 32),    // Casi negro
            surface: Color32::from_rgb(45, 45, 45),       // Gris oscuro
            surface_variant: Color32::from_rgb(55, 55, 55),
            outline: Color32::from_rgb(100, 100, 100),
            outline_variant: Color32::from_rgb(70, 70, 70),
            
            success: Color32::from_rgb(16, 124, 16),
            warning: Color32::from_rgb(255, 184, 0),
            error: Color32::from_rgb(255, 0, 0),
            info: Color32::from_rgb(0, 120, 215),
            
            text_primary: Color32::from_rgb(229, 229, 229),
            text_secondary: Color32::from_rgb(179, 179, 179),
            text_tertiary: Color32::from_rgb(128, 128, 128),
            text_disabled: Color32::from_rgb(80, 80, 80),
            
            interactive_primary: Color32::from_rgb(0, 120, 215),
            interactive_hover: Color32::from_rgb(30, 140, 235),
            interactive_pressed: Color32::from_rgb(0, 90, 180),
            interactive_disabled: Color32::from_rgb(80, 80, 80),
        }
    }
    
    /// Tema claro
    pub fn light() -> Self {
        Self {
            primary: Color32::from_rgb(0, 120, 215),
            secondary: Color32::from_rgb(100, 100, 100),
            background: Color32::from_rgb(242, 242, 242),
            surface: Color32::from_rgb(255, 255, 255),
            surface_variant: Color32::from_rgb(245, 245, 245),
            outline: Color32::from_rgb(200, 200, 200),
            outline_variant: Color32::from_rgb(220, 220, 220),
            
            success: Color32::from_rgb(16, 124, 16),
            warning: Color32::from_rgb(255, 184, 0),
            error: Color32::from_rgb(255, 0, 0),
            info: Color32::from_rgb(0, 120, 215),
            
            text_primary: Color32::from_rgb(0, 0, 0),
            text_secondary: Color32::from_rgb(80, 80, 80),
            text_tertiary: Color32::from_rgb(128, 128, 128),
            text_disabled: Color32::from_rgb(190, 190, 190),
            
            interactive_primary: Color32::from_rgb(0, 120, 215),
            interactive_hover: Color32::from_rgb(30, 140, 235),
            interactive_pressed: Color32::from_rgb(0, 90, 180),
            interactive_disabled: Color32::from_rgb(190, 190, 190),
        }
    }
}

/// Constantes de espaciado y dimensiones Fluent
pub mod spacing {
    use egui::Vec2;
    
    pub const NONE: f32 = 0.0;
    pub const EXTRA_SMALL: f32 = 4.0;
    pub const SMALL: f32 = 8.0;
    pub const MEDIUM: f32 = 12.0;
    pub const LARGE: f32 = 16.0;
    pub const EXTRA_LARGE: f32 = 24.0;
    pub const XXL: f32 = 32.0;
    
    pub fn compact() -> Vec2 {
        Vec2::new(SMALL, SMALL)
    }
    
    pub fn normal() -> Vec2 {
        Vec2::new(MEDIUM, MEDIUM)
    }
    
    pub fn comfortable() -> Vec2 {
        Vec2::new(LARGE, LARGE)
    }
}

/// Estilos de bordes redondeados Fluent
pub mod rounding {
    use egui::Rounding;
    
    pub fn none() -> Rounding {
        Rounding::ZERO
    }
    
    pub fn small() -> Rounding {
        Rounding::same(4.0)
    }
    
    pub fn medium() -> Rounding {
        Rounding::same(8.0)
    }
    
    pub fn large() -> Rounding {
        Rounding::same(12.0)
    }
    
    pub fn circle() -> Rounding {
        Rounding::same(100.0)
    }
}

/// Estilos de líneas
pub mod stroke {
    use egui::{Stroke, Color32};
    
    pub fn thin(color: Color32) -> Stroke {
        Stroke::new(0.5, color)
    }
    
    pub fn normal(color: Color32) -> Stroke {
        Stroke::new(1.0, color)
    }
    
    pub fn thick(color: Color32) -> Stroke {
        Stroke::new(2.0, color)
    }
    
    pub fn extra_thick(color: Color32) -> Stroke {
        Stroke::new(3.0, color)
    }
}

/// Estilos para diferentes estados de componentes
#[derive(Clone, Copy, Debug)]
pub enum ComponentState {
    Default,
    Hovered,
    Pressed,
    Disabled,
    Focused,
}

impl ComponentState {
    pub fn opacity(self) -> f32 {
        match self {
            ComponentState::Disabled => 0.5,
            _ => 1.0,
        }
    }
}

/// Configuración de tipografía
pub mod typography {
    /// Tamaños de fuente en puntos
    pub const CAPTION: f32 = 12.0;
    pub const BODY: f32 = 14.0;
    pub const BODY_STRONG: f32 = 14.0;
    pub const SUBTITLE: f32 = 16.0;
    pub const TITLE: f32 = 20.0;
    pub const TITLE_LARGE: f32 = 28.0;
    pub const DISPLAY: f32 = 40.0;
}

/// Sombras en Fluent Design
pub struct Shadow {
    pub blur_radius: f32,
    pub spread_radius: f32,
    pub offset: (f32, f32),
    pub color: Color32,
}

impl Shadow {
    pub fn none() -> Self {
        Self {
            blur_radius: 0.0,
            spread_radius: 0.0,
            offset: (0.0, 0.0),
            color: Color32::TRANSPARENT,
        }
    }
    
    pub fn subtle(_theme: &FluentTheme) -> Self {
        Self {
            blur_radius: 4.0,
            spread_radius: 0.0,
            offset: (0.0, 1.0),
            color: Color32::from_black_alpha(20),
        }
    }
    
    pub fn medium(theme: &FluentTheme) -> Self {
        Self {
            blur_radius: 8.0,
            spread_radius: 0.0,
            offset: (0.0, 2.0),
            color: Color32::from_black_alpha(40),
        }
    }
    
    pub fn elevation_16(theme: &FluentTheme) -> Self {
        Self {
            blur_radius: 16.0,
            spread_radius: 0.0,
            offset: (0.0, 4.0),
            color: Color32::from_black_alpha(60),
        }
    }
}
