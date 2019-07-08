use serde::{Serialize, Deserialize};
use std::ops::{Range, Sub, Add, Deref, DerefMut};
use std::rc::Rc;

#[derive(Serialize, Deserialize, Debug)]
pub struct Area {
    north: Option<NeighborArea>,
    south: Option<NeighborArea>,
    east: Option<NeighborArea>,
    west: Option<NeighborArea>,

    // map properties
    tilemap: TileMap,
    pub name: String, // must be unique
}

impl Deref for Area{
    type Target = TileMap;

    fn deref(&self) -> &Self::Target {
        &self.tilemap
    }
}

impl DerefMut for Area{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tilemap
    }
}

impl From<Vec<Vec<u32>>> for Area {
    fn from(map: Vec<Vec<u32>>) -> Self {
        Area {
            tilemap: TileMap::from(map),
            ..Area::default()
        }
    }
}


impl From<Vec<Vec<u32>>> for TileMap {
    fn from(map: Vec<Vec<u32>>) -> Self {
        let mut flat_map = vec![];

        let height = map.len();
        let width = {
            if height > 0 {
                map[0].len()
            } else {
                0
            }
        };

        for y in map.iter() {
            for x in y.iter() {
                flat_map.push(*x);
            }
        }

        TileMap {
            width,
            height,
            map: flat_map
        }
    }
}

impl PartialEq for TileMap {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width && self.height == other.height && self.map == other.map
    }
}

impl Area {
    pub fn default() -> Self {
        Area {
            north: None,
            south: None,
            east: None,
            west: None,
            tilemap: TileMap::from(vec![]),
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
            tilemap: TileMap::new(width, height, new_map),
            ..Area::default()
        }
    }

    pub fn set_tile(&mut self, x: usize, y: usize, value: u32) {
        let index = y*self.width + x;
        self.map[index] = value;
    }

    pub fn join_maps(&self, north: Option<&Self>, south: Option<&Self>, east: Option<&Self>, west: Option<&Self>) -> Self {
        // TODO: create lib for integral cartesian bullshit
        let center_origin = Point::new(0, 0);
        let center_end = center_origin + Point::new(self.width as i32, self.height as i32);

        // do this stuff
        let (north_origin, north_end) = match (north, &self.north) {
            (Some(north), Some(neighbor)) => { (
                Point{ x: center_origin.x + neighbor.offset, y: center_origin.y - north.height as i32},
                Point{ x: north.width as i32 + neighbor.offset, y: center_origin.y }
            )},
            (Some(_), None) => { panic!("illegal neighbor to the north") },
            _ => (Point {x: 0,y: 0}, Point {x: 0, y: 0})
        };

        let (south_origin, south_end) = match (south, &self.south) {
            (Some(south), Some(neighbor)) => {(
                Point{ x: center_origin.x + neighbor.offset, y: center_end.y },
                Point{ x: south.width as i32 + neighbor.offset, y: south.height as i32 + center_end.y }
            )},
            (Some(_), None) => { panic!("illegal neighbor to the north") },
            _ => (Point {x: 0,y: 0}, Point {x: 0, y: 0})
        };


        let (west_origin, west_end) = match (west, &self.west) {
            (Some(west), Some(neighbor)) => { (
                Point{ x: center_origin.x - west.width as i32, y: center_origin.y + neighbor.offset},
                Point{ x: center_origin.x, y: west.height as i32 + neighbor.offset }
            )},
            (Some(_), None) => { panic!("illegal neighbor to the west") },
            _ => (Point {x: 0,y: 0}, Point {x: 0, y: 0})
        };

        let (east_origin, east_end) = match (east, &self.east) {
            (Some(east), Some(neighbor)) => {(
                Point{ x: center_end.x, y: center_origin.y + neighbor.offset },
                Point{ x: east.width as i32 + center_end.x, y: east.height as i32 + neighbor.offset }
            )},
            (Some(_), None) => { panic!("illegal neighbor to the north") },
            _ => (Point {x: 0,y: 0}, Point {x: 0, y: 0})
        };

        // find new bounds
        use std::cmp::{min, max};
        let min_x = min( east_origin.x, min(south_origin.x, min(west_origin.x, min(center_origin.x, north_origin.x))));
        let min_y = min( east_origin.y, min(south_origin.y, min(west_origin.y, min(center_origin.y, north_origin.y))));

        let area_offsets = Point::new(min_x, min_y);

        let max_x = max(east_end.x, max(south_end.x, max(west_end.x, max(center_end.x, north_end.x))));
        let max_y = max(east_end.y, max(south_end.y, max(west_end.y, max(center_end.y, north_end.y))));
        let area_size = Point::new(max_x, max_y) - area_offsets;

        // align sections
        let (north_origin, north_end) = (north_origin - area_offsets, north_end - area_offsets);
        let (west_origin, west_end) = (west_origin - area_offsets, west_end - area_offsets);
        let (center_origin, center_end) = (center_origin - area_offsets, center_end - area_offsets);
        let (east_origin, east_end) = (east_origin - area_offsets, east_end - area_offsets);
        let (south_origin, south_end) = (south_origin - area_offsets, south_end - area_offsets);


        // FUCK
        let mut map: Vec<u32> = vec![];

        fn fuck(m: &Area, start: Point, origin: Point, map: &mut Vec<u32>, area_width: i32) -> Range<i32> {
            let w  = m.width as i32;
            let (x, y) = {
                let xy = start - origin;
                (xy.x, xy.y)
            };
            map.extend(&m.map[(y*w) as usize..(y*w+w) as usize]);
            (start.x + w)..area_width
        }

        // do the hard stuff
        let mut y_iter = 0..area_size.y;
        while let Some(y) = y_iter.next() {
            let mut x_iter = 0..area_size.x;
            while let Some(x) = x_iter.next() {
                let pt = Point {x, y};

                if pt.between(north_origin, north_end) && north.is_some() {
                    x_iter = fuck(north.unwrap(), pt, north_origin, &mut map, area_size.x);
                } else if pt.between(west_origin, west_end) && west.is_some() {
                    x_iter = fuck(west.unwrap(), pt, west_origin, &mut map, area_size.x);
                } else if pt.between(center_origin, center_end) {
                    x_iter = fuck(self, pt, center_origin, &mut map, area_size.x);
                } else if pt.between(east_origin, east_end) && east.is_some()  {
                    x_iter = fuck(east.unwrap(),pt, east_origin, &mut map, area_size.x);
                } else if pt.between(south_origin, south_end) && south.is_some()  {
                    x_iter = fuck(south.unwrap(), pt, south_origin, &mut map, area_size.x);
                } else {
                    map.push(0);
                }
            }
        }

        Area {
            tilemap: TileMap {
                width: area_size.x as usize,
                height: area_size.y as usize,
                map
            },
            ..Area::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct NeighborArea {
    offset: i32,
    area: String
}



#[derive(Serialize, Deserialize, Debug)]
pub struct TileMap {
    pub width: usize,
    pub height: usize,
    pub map: Vec<u32>,
}

impl TileMap {
    fn new(width: usize, height: usize, map: Vec<u32>) -> TileMap {
        TileMap {
            width,
            height,
            map
        }
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::rc::Rc;

    fn new_areas_dict() -> HashMap<String, Rc<Area>> {
        HashMap::new()
    }

    fn prep_area(hashmap: &mut HashMap<String, Rc<Area>>, deserialize: &str) -> Rc<Area> {
        let area: Rc<Area> = Rc::new(serde_json::from_str(deserialize).unwrap());
        hashmap.insert(area.name.clone(), area.clone());
        area
    }

    #[test]
    fn create_from_ivec2() {
        let ctr = Area {
            tilemap: TileMap::from(vec![vec![1,2], vec![3,4]]),
            ..Area::default()
        };
        assert_eq!(ctr.map, vec![1,2,3,4]);
        assert_eq!(ctr.width, 2);
        assert_eq!(ctr.height, 2);
    }

    #[test]
    fn join_left() {
        let mut areas = new_areas_dict();

        let ctr = prep_area(&mut areas, r#"
            {
              "west": {
                "offset": 0,
                "area": "left"
              },
              "tilemap": {
                "width": 2,
                "height": 2,
                "map": [
                  1, 2,
                  3, 4
                ]
              },
              "name": "center"
            }
        "#);

        let left = prep_area(&mut areas,r#"
            {
              "east": {
                "offset": 0,
                "area": "center"
              },
              "tilemap": {
                "width": 2,
                "height": 2,
                "map": [
                  4, 4,
                  5, 6
                ]
              },
              "name": "left"
            }
        "#);

        let ww3 = ctr.west.clone().unwrap().area;
        let lol = areas.get(ww3.as_str()).unwrap().to_owned();


        let joined: Area = ctr.join_maps(None, None, None, Some(&lol));
        let joined_map = joined.tilemap;

        let expected: TileMap = serde_json::from_str(r#"
            {
              "width": 4,
              "height": 2,
              "map": [
                4, 4, 1, 2,
                5, 6, 3, 4
              ]
            }
        "#).unwrap();

        assert_eq!(joined_map, expected);
    }

    #[test]
    fn simple_overflow_left() {
        let mut areas = new_areas_dict();

        let ctr = prep_area(&mut areas, r#"
            {
              "west": {
                "offset": 0,
                "area": "left"
              },
              "tilemap": {
                "width": 1,
                "height": 1,
                "map": [ 1 ]
              },
              "name": "center"
            }
        "#);

        let left = prep_area(&mut areas,r#"
            {
              "east": {
                "offset": 0,
                "area": "center"
              },
              "tilemap": {
                "width": 1,
                "height": 2,
                "map": [
                  4,
                  5
                ]
              },
              "name": "left"
            }
        "#);

        let ww3 = ctr.west.clone().unwrap().area;
        let lol = areas.get(ww3.as_str()).unwrap().to_owned();


        let joined: Area = ctr.join_maps(None, None, None, Some(&lol));
        let joined_map = joined.tilemap;

        let expected: TileMap = serde_json::from_str(r#"
            {
              "width": 2,
              "height": 2,
              "map": [
                4, 1,
                5, 0
              ]
            }
        "#).unwrap();

        assert_eq!(joined_map, expected);
    }

    #[test]
    fn complex_join() {
        let mut areas = new_areas_dict();

        let center = prep_area(&mut areas, r#"{
            "east": {
                "offset": -1,
                "area": "right"
            },
            "west": {
                "offset": 0,
                "area": "left"
            },
            "south": {
                "offset": -1,
                "area": "down"
            },
            "tilemap": {
                "width": 2,
                "height": 2,
                "map": [2, 2, 2, 2]
            },
            "name": "center"
        }"#);

        let left = prep_area(&mut areas, r#"{
            "tilemap": {
                "width": 2,
                "height": 2,
                "map": [1, 1, 1, 1]
            },
            "name": "left"
        }"#);




        let right = prep_area(&mut areas, r#"{
            "tilemap": {
                "width": 2,
                "height": 3,
                "map": [3, 3, 3, 3, 3, 3]
            },
            "name": "right"
        }"#);


        let down = prep_area(&mut areas, r#"{
            "tilemap": {
                "width": 4,
                "height": 2,
                "map": [4, 4, 4, 4, 4, 4, 4, 4]
            },
            "name": "down"
        }"#);

        let joined: Area = center.join_maps(None, Some(&down), Some(&right), Some(&left));

        let expected = prep_area(&mut areas, r#"{
            "tilemap": {
                "width": 6,
                "height": 5,
                "map": [0, 0, 0, 0, 3, 3, 1, 1, 2, 2, 3, 3, 1, 1, 2, 2, 3, 3, 0, 4, 4, 4, 4, 0, 0, 4, 4, 4, 4, 0]
            },
            "name": ""
        }"#);

        assert_eq!(joined.tilemap, expected.tilemap);
    }
}

















#[derive(Debug, Copy, Clone)]
struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point {x, y}
    }

    fn between(&self, p1: Point, p2: Point) -> bool {
        self.x >= p1.x && self.y >= p1.y && p2.x > self.x && self.y < p2.y ||
            p2.x <= self.x && self.y >= p2.y && self.x < p1.x && self.y < p1.y
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