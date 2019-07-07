use serde::{Serialize, Deserialize};
use std::ops::{Range, Sub, Add};

#[derive(Serialize, Deserialize, Debug)]
pub struct Area {
    // TODO: Cardinal direction areas
    north: Option<NeighborArea>,
    south: Option<NeighborArea>,
    east: Option<NeighborArea>,
    west: Option<NeighborArea>,

    // map properties
    pub width: usize,
    pub height: usize,
    pub map: Vec<u32>,
    pub name: String, // must be unique
}

impl From<Vec<Vec<u32>>> for Area {
    fn from(map: Vec<Vec<u32>>) -> Self {
        let mut flat_map = vec![];

        let height = map.len();
        let width = map[0].len();

        for y in map.iter() {
            for x in y.iter() {
                flat_map.push(*x);
            }
        }

        Area {
            width,
            height,
            map: flat_map,
            ..Area::default()
        }
    }
}

impl Area {
    pub fn default() -> Self {
        Area {
            north: None,
            south: None,
            east: None,
            west: None,
            width: 0,
            height: 0,
            map: vec![],
            name: "".to_string()
        }
    }

    pub fn view_slice(&self, x_range: Range<usize>, y_range: Range<usize>) -> Self {
        let (width, height) = (x_range.len(), y_range.len());

        let mut new_map = vec![];

        let y = y_range.start;
        let take_y = if y > self.height { 0 } else { self.height - y };
        let take_y = if take_y > height { height } else { take_y };
        let leftover_y = height - take_y;

        for y_offset in y..y+take_y {
            let literal_offset = y_offset * self.width;
            let x = x_range.start;
            let take_x = if x > self.width { 0 } else { self.width - x };
            let take_x = if take_x > width { width } else { take_x };
            let leftover_x = width - take_x;

            if take_x > 0 {
                new_map.extend_from_slice(&self.map[literal_offset+x..take_x+literal_offset+x]);
            }
            new_map.extend(vec![0u32; leftover_x])
        }
        new_map.extend(vec![0u32; leftover_y * width]);

        Area {
            width,
            height,
            map: new_map,
            ..Area::default()
        }
    }

    pub fn set_tile(&mut self, x: usize, y: usize, value: u32) {
        self.map[y*self.width + x] = value;
    }

    pub fn join_maps(&self, north: Option<&Self>, south: Option<&Self>, east: Option<&Self>, west: Option<&Self>) -> Self {
        // TODO: create lib for integral cartesian bullshit

        let center_origin = Point::new(0, 0);
        let center_ends = center_origin + Point::new(self.width as i32, self.height as i32);

        let (north_origin, north_ends) = {
            (center_origin, center_ends)
        };

        Area {
            width: 0,
            height: 0,
            map: vec![],
            ..Area::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy)]
struct NeighborArea {
    offset: i32,
    area: String
}




















struct Point {
    pub x: i32,
    pub y: i32,
};

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point {x, y}
    }

    fn between(&self, p1: Point, p2: Point) -> bool {
        false
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {x: self.x + other.x, y: self.y + other.y}
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Point {
        Point {x: self.x - other.x, y: self.y - other.y}
    }
}