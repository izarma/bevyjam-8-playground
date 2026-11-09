use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use bevy::prelude::*;
use plants::Plant;
use rand::seq::IndexedRandom;

use super::{DialogueTarget, YarnSpinnerDialogueViewSystemSet, view::DialogueOptionsNode};
use crate::{
    gameplay::{
        clock::Clock,
        dialogue::herb_drag::{HerbPicker, Ingredient, ingredient},
        herbs::{self, HeldCount},
        inventory::Inventory,
        upgrades::{Rewards, Trader},
    },
    ui::{
        alerts::Announce,
        font,
        interaction::InteractionPalette,
        palette::{BUTTON_HOVERED_TINT, BUTTON_PRESSED_TINT, HEADER_TEXT, LABEL_TEXT},
        scale::texels,
        widget,
    },
};

const GIVE: usize = 2;
const HERB_ROWS: u16 = 3;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(release_dialogue).add_systems(
        Update,
        show_trade
            .in_set(YarnSpinnerDialogueViewSystemSet)
            .run_if(any_with_component::<TradeWindow>),
    );
}

#[derive(Component)]
struct TradeWindow {
    trader: Entity,
    give: [Option<Plant>; GIVE],
    get: [Plant; GIVE],
    choosing: bool,
    done: Arc<AtomicBool>,
}

impl TradeWindow {
    fn available(&self, inventory: &Inventory, plant: Plant) -> u32 {
        let held = inventory.plants.get(&plant).copied().unwrap_or(0);
        let given = self
            .give
            .iter()
            .filter(|&&given| given == Some(plant))
            .count() as u32;
        held.saturating_sub(given)
    }
}

fn tradeable() -> Vec<Plant> {
    herbs::shifting().map(|(plant, _)| plant).collect()
}

pub(super) fn open_trade(
    mut commands: Commands,
    target: Res<DialogueTarget>,
    options: Query<Entity, With<DialogueOptionsNode>>,
    inventory: Single<&Inventory>,
    rewards: Res<Rewards>,
) -> Arc<AtomicBool> {
    let (Some(trader), Some(options)) = (target.0, options.iter().next()) else {
        error!("open_trade ran without a trader in dialogue");
        return Arc::new(AtomicBool::new(true));
    };
    let done = Arc::new(AtomicBool::new(false));
    let held = inventory
        .plants
        .iter()
        .map(|(&plant, &count)| (plant, count));
    let give = (0..GIVE)
        .map(|slot| -> Box<dyn Scene> { Box::new(trade_slot(true, slot)) })
        .collect::<Vec<_>>();
    let get = (0..GIVE)
        .map(|slot| -> Box<dyn Scene> { Box::new(trade_slot(false, slot)) })
        .collect::<Vec<_>>();
    let all = tradeable();
    commands
        .spawn_scene(bsn! {
            #Trade
            ChildOf(options)
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(texels(5)),
                row_gap: texels(4),
            }
            @widget::panel()
            Children [
                TradeStock
                Text("")
                @font::text(1)
                TextColor(HEADER_TEXT)
                Pickable::IGNORE
                --
                Node {
                    column_gap: texels(2),
                    align_items: AlignItems::Center,
                }
                Children [
                    { give }
                    --
                    Text("for")
                    @font::text(1)
                    TextColor(LABEL_TEXT)
                    Node {
                        margin: UiRect::horizontal(texels(3)),
                    }
                    Pickable::IGNORE
                    --
                    { get }
                    --
                    Node {
                        margin: UiRect::left(texels(4)),
                    }
                    Children [ @widget::button("Trade", trade) ]
                ]
                --
                Node {
                    height: { herbs::grid_height(HERB_ROWS) },
                    align_items: AlignItems::FlexStart,
                    overflow: Overflow::clip(),
                }
                Children [ @herbs::grid(held, ingredient::<TradeWindow>) ]
            ]
        })
        .insert(TradeWindow {
            trader,
            give: [None; GIVE],
            get: [all[0], all[1]],
            choosing: rewards.trader_chooses(),
            done: done.clone(),
        });
    done
}

#[derive(Component, Clone, Copy, Default, PartialEq, Eq)]
struct TradeSlot(bool, usize);

#[derive(Component, Clone, Default)]
struct TradeIcon;

#[derive(Component, Clone, Default)]
struct TradeUnknown;

#[derive(Component, Clone, Default)]
struct TradeStock;

fn trade_slot(giving: bool, index: usize) -> impl Scene {
    bsn! {
        Node {
            width: texels(24),
            height: texels(24),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        @widget::panel()
        TradeSlot(giving, index)
        InteractionPalette {
            none: Color::WHITE,
            hovered: BUTTON_HOVERED_TINT,
            pressed: BUTTON_PRESSED_TINT,
        }
        @on(click_slot)
        @on(drop_herb)
        Children [
            @herbs::icon(Plant::default())
            TradeIcon
            Node {
                display: Display::None,
            }
            --
            TradeUnknown
            Text("?")
            @font::text(1)
            TextColor(LABEL_TEXT)
            Node {
                display: Display::None,
            }
            Pickable::IGNORE
        ]
    }
}

impl HerbPicker for TradeWindow {
    fn available(&self, inventory: &Inventory, plant: Plant) -> u32 {
        TradeWindow::available(self, inventory, plant)
    }
}

fn drop_herb(
    drop: On<PointerDragDrop>,
    slots: Query<&TradeSlot>,
    ingredients: Query<&Ingredient>,
    mut window: Single<&mut TradeWindow>,
    inventory: Single<&Inventory>,
) {
    let (Ok(&TradeSlot(true, slot)), Ok(&Ingredient(plant))) =
        (slots.get(drop.entity), ingredients.get(drop.dropped))
    else {
        return;
    };
    let freed = u32::from(window.give[slot] == Some(plant));
    if window.available(&inventory, plant) + freed > 0 {
        window.give[slot] = Some(plant);
    }
}

fn click_slot(
    click: On<PointerClick>,
    slots: Query<&TradeSlot>,
    mut window: Single<&mut TradeWindow>,
) {
    match slots.get(click.entity) {
        Ok(&TradeSlot(true, slot)) => window.give[slot] = None,
        Ok(&TradeSlot(false, slot)) if window.choosing => {
            let all = tradeable();
            let at = all.iter().position(|&plant| plant == window.get[slot]);
            window.get[slot] = all[at.map_or(0, |at| (at + 1) % all.len())];
        }
        _ => {}
    }
}

fn trade(
    _: On<PointerClick>,
    mut commands: Commands,
    window: Single<&TradeWindow>,
    mut traders: Query<&mut Trader>,
    mut inventory: Single<&mut Inventory>,
    clock: Single<&Clock>,
) {
    let Ok(mut trader) = traders.get_mut(window.trader) else {
        return;
    };
    let ([Some(first), Some(second)], true) = (window.give, trader.stock > 0) else {
        return;
    };
    for plant in [first, second] {
        if let Some(count) = inventory.plants.get_mut(&plant) {
            *count -= 1;
            if *count == 0 {
                inventory.plants.remove(&plant);
            }
        }
    }
    let got = if window.choosing {
        window.get
    } else {
        let all = tradeable();
        let mut rng = rand::rng();
        [0; GIVE].map(|_| *all.choose(&mut rng).expect("herbs to trade for"))
    };
    for plant in got {
        *inventory.plants.entry(plant).or_default() += 1;
    }
    trader.traded(clock.hours);
    commands.trigger(Announce(format!(
        "You traded for {} and {}.",
        herbs::name(got[0]),
        herbs::name(got[1])
    )));
    window.done.store(true, Ordering::Relaxed);
}

fn release_dialogue(remove: On<Remove<TradeWindow>>, windows: Query<&TradeWindow>) {
    if let Ok(window) = windows.get(remove.entity) {
        window.done.store(true, Ordering::Relaxed);
    }
}

fn show_trade(
    window: Single<&TradeWindow>,
    traders: Query<&Trader>,
    inventory: Single<&Inventory>,
    slots: Query<(&TradeSlot, &Children)>,
    mut icons: Query<(&mut ImageNode, &mut Node), With<TradeIcon>>,
    mut unknowns: Query<&mut Node, (With<TradeUnknown>, Without<TradeIcon>)>,
    mut stock: Single<&mut Text, With<TradeStock>>,
    offerings: Query<&Children, With<Ingredient>>,
    mut counts: Query<(&HeldCount, &mut Text), Without<TradeStock>>,
) {
    let left = traders.get(window.trader).map_or(0, |trader| trader.stock);
    let shown = match left {
        0 => "No trades left today".to_string(),
        1 => "1 trade left today".to_string(),
        n => format!("{n} trades left today"),
    };
    if stock.0 != shown {
        stock.0 = shown;
    }
    for (&slot, children) in &slots {
        let TradeSlot(giving, index) = slot;
        let herb = if giving {
            window.give[index]
        } else {
            window.choosing.then_some(window.get[index])
        };
        let unknown = !giving && !window.choosing;
        for &child in children {
            if let Ok((mut icon, mut node)) = icons.get_mut(child) {
                if let Some(plant) = herb {
                    let rect = Some(herbs::icon_rect(plant));
                    if icon.rect != rect {
                        icon.rect = rect;
                    }
                }
                show(&mut node, herb.is_some());
            }
            if let Ok(mut node) = unknowns.get_mut(child) {
                show(&mut node, unknown);
            }
        }
    }
    for &child in offerings.iter().flatten() {
        if let Ok((&HeldCount(plant), mut text)) = counts.get_mut(child) {
            let shown = window.available(&inventory, plant).to_string();
            if text.0 != shown {
                text.0 = shown;
            }
        }
    }
}

fn show(node: &mut Node, visible: bool) {
    let display = if visible {
        Display::Flex
    } else {
        Display::None
    };
    if node.display != display {
        node.display = display;
    }
}
