use kay::{ActorSystem, World, External, TypedID};
use imgui::Ui;
use core::simulation::{TimeOfDay, TimeOfDayRange, Duration};
use economy::resources::Resource;
use economy::market::{Deal, OfferID, EvaluationRequester, EvaluationRequesterID,
                      EvaluatedSearchResult, MSG_EvaluationRequester_expect_n_results,
                      MSG_EvaluationRequester_on_result};
use economy::buildings::BuildingID;
use economy::buildings::rendering::BuildingInspectorID;
use transport::pathfinding::RoughLocationID;

use super::{Household, HouseholdID, HouseholdCore, MemberIdx, MSG_Household_decay,
            MSG_Household_inspect, MSG_Household_provide_deal, MSG_Household_receive_deal,
            MSG_Household_task_succeeded, MSG_Household_task_failed, MSG_Household_destroy,
            MSG_Household_stop_using, MSG_Household_reset_member_task};

#[derive(Compact, Clone)]
pub struct GroceryShop {
    id: GroceryShopID,
    site: BuildingID,
    core: HouseholdCore,
    grocery_offer: OfferID,
    job_offer: OfferID,
}

impl GroceryShop {
    pub fn move_into(id: GroceryShopID, site: BuildingID, world: &mut World) -> GroceryShop {
        GroceryShop {
            id,
            site,
            core: HouseholdCore::new(0, site.into()),
            grocery_offer: OfferID::register(
                id.into(),
                MemberIdx(0),
                site.into(),
                TimeOfDayRange::new(7, 0, 20, 0),
                Deal::new(
                    vec![(Resource::Groceries, 30.0), (Resource::Money, -60.0)],
                    Duration::from_minutes(30),
                ),
                world,
            ),
            job_offer: OfferID::register(
                id.into(),
                MemberIdx(0),
                site.into(),
                TimeOfDayRange::new(7, 0, 15, 0),
                Deal::new(Some((Resource::Money, 50.0)), Duration::from_hours(5)),
                world,
            ),
        }
    }
}

impl Household for GroceryShop {
    fn core(&self) -> &HouseholdCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut HouseholdCore {
        &mut self.core
    }

    fn is_shared(_: Resource) -> bool {
        true
    }

    fn supplier_shared(_: Resource) -> bool {
        true
    }

    fn importance(_: Resource, _: TimeOfDay) -> f32 {
        1.0
    }

    fn interesting_resources() -> &'static [Resource] {
        &[Resource::Money, Resource::Groceries]
    }

    fn receive_deal(&mut self, deal: &Deal, _member: MemberIdx, _: &mut World) {
        deal.delta.give_to(&mut self.resources);
    }

    fn provide_deal(&mut self, deal: &Deal, _member: MemberIdx, _: &mut World) {
        deal.delta.take_from(&mut self.resources);
    }

    fn task_succeeded(&mut self, _member: MemberIdx, _: &mut World) {
        unimplemented!()
    }

    fn task_failed(&mut self, _member: MemberIdx, _location: RoughLocationID, _: &mut World) {
        unimplemented!()
    }

    fn reset_member_task(&mut self, _member: MemberIdx, _: &mut World) {
        unimplemented!()
    }

    fn stop_using(&mut self, _offer: OfferID, _: &mut World) {
        unimplemented!()
    }

    fn decay(&mut self, dt: Duration, _: &mut World) {
        let groceries = self.resources.mut_entry_or(Resource::Groceries, 0.0);
        *groceries += 0.001 * dt.as_seconds();
    }

    fn destroy(&mut self, world: &mut World) {
        self.site.remove_household(self.id_as(), world);
        self.grocery_offer.withdraw(world);
        self.job_offer.withdraw(world);
    }

    #[allow(useless_format)]
    fn inspect(
        &mut self,
        imgui_ui: &External<Ui<'static>>,
        return_to: BuildingInspectorID,
        world: &mut World,
    ) {
        let ui = imgui_ui.steal();

        ui.window(im_str!("Building")).build(|| {
            ui.tree_node(im_str!("Grocery Shop RawID: {:?}", self.id.as_raw()))
                .build(|| for resource in Self::interesting_resources() {
                    if Self::is_shared(*resource) {
                        ui.text(im_str!("{}", resource));
                        ui.same_line(100.0);
                        let amount = self.resources.get(*resource).cloned().unwrap_or(0.0);
                        ui.text(im_str!("{:.2}", amount));
                    }
                });
        });

        return_to.ui_drawn(ui, world);
    }
}

impl EvaluationRequester for GroceryShop {
    fn expect_n_results(&mut self, _r: Resource, _n: usize, _: &mut World) {}
    fn on_result(&mut self, _e: &EvaluatedSearchResult, _: &mut World) {}
}

pub fn setup(system: &mut ActorSystem) {
    system.register::<GroceryShop>();
    auto_setup(system);
}

mod kay_auto;
pub use self::kay_auto::*;
