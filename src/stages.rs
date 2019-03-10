use map::{Block, BlockAndPosition, Map, Size, Stage};
use quicksilver::{geom::Vector, graphics::Color};
use Position;

/*#[allow(dead_code)]*/
//pub fn get_stages() -> Vec<Stage> {
//vec![
//Stage {
//stage: 1,
//maps: vec![
//Map {
//level: 1,
//time: 300_000,
//blocks_with_position: vec![
//BlockAndPosition {
//block: Block {
//can_be_moved: false,
//size: Size {
//width: 50.0,
//height: 50.0,
//},
//color: Color {
//r: 1.0,
//g: 1.0,
//b: 0.0,
//a: 1.0,
//},
//},
//position: Position(Vector { x: 50.0, y: 50.0 }),
//},
//BlockAndPosition {
//block: Block {
//can_be_moved: true,
//size: Size {
//width: 50.0,
//height: 50.0,
//},
//color: Color {
//r: 1.0,
//g: 0.0,
//b: 1.0,
//a: 1.0,
//},
//},
//position: Position(Vector { x: 50.0, y: 100.0 }),
//},
//BlockAndPosition {
//block: Block {
//can_be_moved: true,
//size: Size {
//width: 50.0,
//height: 50.0,
//},
//color: Color {
//r: 1.0,
//g: 0.0,
//b: 1.0,
//a: 1.0,
//},
//},
//position: Position(Vector { x: 50.0, y: 150.0 }),
//},
//BlockAndPosition {
//block: Block {
//can_be_moved: true,
//size: Size {
//width: 50.0,
//height: 50.0,
//},
//color: Color {
//r: 1.0,
//g: 0.0,
//b: 1.0,
//a: 1.0,
//},
//},
//position: Position(Vector { x: 50.0, y: 200.0 }),
//},
//BlockAndPosition {
//block: Block {
//can_be_moved: true,
//size: Size {
//width: 50.0,
//height: 50.0,
//},
//color: Color {
//r: 1.0,
//g: 0.0,
//b: 1.0,
//a: 1.0,
//},
//},
//position: Position(Vector { x: 50.0, y: 250.0 }),
//},
//],
//},
//Map {
//level: 2,
//time: 300_000,
//blocks_with_position: vec![
//BlockAndPosition {
//block: Block {
//can_be_moved: false,
//size: Size {
//width: 50.0,
//height: 50.0,
//},
//color: Color {
//r: 1.0,
//g: 0.5,
//b: 0.0,
//a: 1.0,
//},
//},
//position: Position(Vector { x: 150.0, y: 150.0 }),
//},
//BlockAndPosition {
//block: Block {
//can_be_moved: true,
//size: Size {
//width: 50.0,
//height: 50.0,
//},
//color: Color {
//r: 0.5,
//g: 0.0,
//b: 1.0,
//a: 1.0,
//},
//},
//position: Position(Vector { x: 150.0, y: 100.0 }),
//},
//],
//},
//],
//},
//Stage {
//stage: 2,
//maps: vec![
//Map {
//level: 1,
//time: 300_000,
//blocks_with_position: vec![
//BlockAndPosition {
//block: Block {
//can_be_moved: false,
//size: Size {
//width: 50.0,
//height: 50.0,
//},
//color: Color {
//r: 1.0,
//g: 1.0,
//b: 0.0,
//a: 1.0,
//},
//},
//position: Position(Vector { x: 50.0, y: 50.0 }),
//},
//BlockAndPosition {
//block: Block {
//can_be_moved: true,
//size: Size {
//width: 50.0,
//height: 50.0,
//},
//color: Color {
//r: 1.0,
//g: 0.0,
//b: 1.0,
//a: 1.0,
//},
//},
//position: Position(Vector { x: 50.0, y: 100.0 }),
//},
//BlockAndPosition {
//block: Block {
//can_be_moved: true,
//size: Size {
//width: 50.0,
//height: 50.0,
//},
//color: Color {
//r: 1.0,
//g: 0.0,
//b: 1.0,
//a: 1.0,
//},
//},
//position: Position(Vector { x: 50.0, y: 150.0 }),
//},
//BlockAndPosition {
//block: Block {
//can_be_moved: true,
//size: Size {
//width: 50.0,
//height: 50.0,
//},
//color: Color {
//r: 1.0,
//g: 0.0,
//b: 1.0,
//a: 1.0,
//},
//},
//position: Position(Vector { x: 50.0, y: 200.0 }),
//},
//BlockAndPosition {
//block: Block {
//can_be_moved: true,
//size: Size {
//width: 50.0,
//height: 50.0,
//},
//color: Color {
//r: 0.0,
//g: 0.0,
//b: 1.0,
//a: 1.0,
//},
//},
//position: Position(Vector { x: 50.0, y: 250.0 }),
//},
//],
//},
//Map {
//level: 2,
//time: 300_000,
//blocks_with_position: vec![
//BlockAndPosition {
//block: Block {
//can_be_moved: false,
//size: Size {
//width: 50.0,
//height: 50.0,
//},
//color: Color {
//r: 1.0,
//g: 0.5,
//b: 0.0,
//a: 1.0,
//},
//},
//position: Position(Vector { x: 150.0, y: 150.0 }),
//},
//BlockAndPosition {
//block: Block {
//can_be_moved: true,
//size: Size {
//width: 50.0,
//height: 50.0,
//},
//color: Color {
//r: 0.5,
//g: 0.0,
//b: 1.0,
//a: 1.0,
//},
//},
//position: Position(Vector { x: 150.0, y: 100.0 }),
//},
//],
//},
//],
//},
//]
/*}*/
