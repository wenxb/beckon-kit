mod support;

use gpui::{PrimitiveBatch, Scene};
use support::scene::{blur, bounds, quad};

fn color(value: u32) -> gpui::Hsla {
    gpui::rgb(value).into()
}

fn emitted_orders(scene: &Scene) -> Vec<u32> {
    scene
        .batches()
        .flat_map(|batch| match batch {
            PrimitiveBatch::Quads(range) => scene.quads[range]
                .iter()
                .map(|quad| quad.order)
                .collect::<Vec<_>>(),
            PrimitiveBatch::BackdropBlurs(range) => scene.backdrop_blurs[range]
                .iter()
                .map(|blur| blur.order)
                .collect(),
            other => panic!("unexpected primitive in quad-only test: {other:?}"),
        })
        .collect()
}

#[test]
fn blur_is_a_global_barrier_even_for_neighbors_only_in_its_halo() {
    let mut scene = Scene::default();
    scene.push_layer(bounds(0.0, 0.0, 300.0, 200.0));
    scene.insert_primitive(quad(bounds(10.0, 30.0, 10.0, 20.0), color(0xffffff)));
    scene.insert_primitive(blur(bounds(40.0, 20.0, 60.0, 60.0), 12.0));
    scene.insert_primitive(quad(bounds(20.0, 30.0, 20.0, 20.0), color(0xff0000)));
    scene.pop_layer();
    scene.insert_primitive(quad(bounds(250.0, 0.0, 10.0, 10.0), color(0x00ff00)));
    scene.finish();
    let order = scene.backdrop_blurs[0].order;
    assert!(scene.quads[0].order < order);
    assert!(scene.quads[1..].iter().all(|quad| quad.order > order));
    assert!(
        emitted_orders(&scene)
            .windows(2)
            .all(|pair| pair[0] <= pair[1])
    );
}

#[test]
fn nested_layers_cannot_move_an_emitted_primitive_past_the_next_blur() {
    let mut scene = Scene::default();
    let full = bounds(0.0, 0.0, 100.0, 100.0);
    scene.push_layer(full);
    scene.insert_primitive(blur(full, 6.0));
    scene.push_layer(full);
    scene.push_layer(full);
    scene.insert_primitive(quad(full, color(0xffffff)));
    scene.pop_layer();
    scene.pop_layer();
    let highest_emitted = scene.quads[0].order;
    scene.insert_primitive(blur(full, 12.0));
    scene.pop_layer();
    scene.finish();
    assert!(scene.backdrop_blurs[1].order > highest_emitted);
    let orders = emitted_orders(&scene);
    assert!(
        orders.windows(2).all(|pair| pair[0] < pair[1]),
        "each barrier needs a distinct phase: {orders:?}"
    );
}

#[test]
fn cached_subtree_replays_the_operation_not_a_previous_frame_snapshot() {
    let full = bounds(0.0, 0.0, 100.0, 100.0);
    let mut previous = Scene::default();
    previous.insert_primitive(quad(full, color(0xff0000)));
    let start = previous.len();
    previous.push_layer(full);
    previous.insert_primitive(blur(full, 8.0));
    previous.push_layer(full);
    previous.insert_primitive(quad(bounds(30.0, 30.0, 10.0, 10.0), color(0xffffff)));
    previous.pop_layer();
    previous.pop_layer();
    previous.finish();

    let mut next = Scene::default();
    next.insert_primitive(quad(full, color(0x00ff00)));
    next.insert_primitive(blur(full, 3.0));
    next.replay(start..previous.len(), &previous);
    next.finish();
    assert_eq!(next.backdrop_blurs.len(), 2);
    assert_eq!(next.backdrop_blurs[1].sigma.0, 8.0);
    assert!(next.backdrop_blurs[1].order > next.backdrop_blurs[0].order);
    assert!(next.quads[1].order > next.backdrop_blurs[1].order);
    assert_eq!(
        next.quads[0].background.as_solid().unwrap(),
        color(0x00ff00)
    );
}

#[test]
fn clearing_a_scene_removes_barriers_and_preserves_normal_batching() {
    let mut scene = Scene::default();
    scene.insert_primitive(blur(bounds(0.0, 0.0, 50.0, 50.0), 10.0));
    scene.clear();
    assert_eq!(scene.len(), 0);
    assert!(scene.backdrop_blurs.is_empty());
    scene.insert_primitive(quad(bounds(0.0, 0.0, 10.0, 10.0), color(0xff0000)));
    scene.insert_primitive(quad(bounds(20.0, 0.0, 10.0, 10.0), color(0x00ff00)));
    scene.finish();
    assert_eq!(emitted_orders(&scene), [1, 1]);
    assert!(
        matches!(scene.batches().next(), Some(PrimitiveBatch::Quads(range)) if range == (0..2))
    );
}

#[test]
fn clipped_out_blur_does_not_split_an_ordinary_batch() {
    let mut scene = Scene::default();
    let mut hidden = blur(bounds(0.0, 0.0, 50.0, 50.0), 10.0);
    hidden.content_mask.bounds = bounds(100.0, 100.0, 10.0, 10.0);
    scene.insert_primitive(hidden);
    scene.insert_primitive(quad(bounds(0.0, 0.0, 10.0, 10.0), color(0xffffff)));
    scene.finish();
    assert!(scene.backdrop_blurs.is_empty());
    assert_eq!(emitted_orders(&scene), [1]);
}
