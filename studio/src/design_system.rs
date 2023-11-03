use bevy::prelude::Color as BevyColor;

pub struct Color;

impl Color {
    // original: rgb(220,  53,  69)
    // brighter: rgb(255,  48,  68)
    pub const RED: BevyColor = BevyColor::rgb(1.000000, 0.188235, 0.266666);
    // original: rgb( 25, 135,  84)
    // brighter: rgb(  0, 187, 100)
    pub const GREEN: BevyColor = BevyColor::rgb(0.000000, 0.733333, 0.392157);
    // original: rgb( 13, 110, 253)
    // brighter: rgb( 47, 131, 255)
    pub const BLUE: BevyColor = BevyColor::rgb(0.184314, 0.513724, 1.000000);
}
