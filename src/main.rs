use std::collections::HashMap;

use iced::executor;
use iced::widget::{Slider, pick_list, text, button, Rule, row, column, container};
use iced::window;
use iced::subscription::Subscription;
use iced::{Alignment, Application, Command, Element, Length, Settings, Theme};
use zeiss_control::{Scope, ScopeTurret};

fn main() {
    /*
    let mut scope = Scope::new(
        "/dev/ttyUSB1",
        "/dev/ttyUSB0"
    ).unwrap();
    */

    ScopeApp::run(Settings::default()).unwrap();
}

struct ScopeApp {
    scope: Option<Scope>,
    ld_value: u8,
    turret_positions: HashMap<ScopeTurret, u8>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Exit,
    ChangeTurret(ScopeTurret, u8),
    LDUpdate(u8),
    UpdateValues,
}

impl Application for ScopeApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut scope = match Scope::new(
            "/dev/ttyUSB1",
            "/dev/ttyUSB0"
        ) {
            Ok(newscope) => {
                Some(newscope)
            },
            Err(errormessage) => {
                println!("ERROR: {}", errormessage);
                None
            }
        };

        let turret_positions = HashMap::from(
            [(ScopeTurret::Objective,
              scope.as_mut().unwrap().turret_pos(ScopeTurret::Objective).unwrap()),
             (ScopeTurret::DensityFilter1,
              scope.as_mut().unwrap().turret_pos(ScopeTurret::DensityFilter1).unwrap()),
             (ScopeTurret::DensityFilter2,
              scope.as_mut().unwrap().turret_pos(ScopeTurret::DensityFilter2).unwrap())]
        );

        (Self {
            scope,
            ld_value: 250,
            turret_positions: turret_positions.clone(),
        }, Command::none())
    }

    fn title(&self) -> String {
        String::from("AxioVision ∞")
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(100)).map(|_| {
            Message::UpdateValues
        })
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Exit => window::close(),
            Message::ChangeTurret(turret, position) => {
                self.scope.as_mut().unwrap().set_turret_pos(turret, position).unwrap();
                self.turret_positions.insert(ScopeTurret::Objective, position);
                Command::none()
            },
            Message::LDUpdate(value) => {
                self.scope.as_mut().unwrap().set_ld_pos(value).unwrap();
                self.ld_value = value;
                Command::none()
            },
            Message::UpdateValues => {
                self.ld_value = self.scope.as_mut().unwrap().ld_pos().unwrap();
                self.turret_positions.insert(
                    ScopeTurret::DensityFilter1,
                    self.scope.as_mut().unwrap().turret_pos(ScopeTurret::DensityFilter1).unwrap()
                );
                self.turret_positions.insert(
                    ScopeTurret::DensityFilter2,
                    self.scope.as_mut().unwrap().turret_pos(ScopeTurret::DensityFilter2).unwrap()
                );
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let objective_turret_selection = column![
            text("Objective").width(Length::Fill),
            row![
                button("1").padding([5, 10])
                    .on_press(Message::ChangeTurret(ScopeTurret::Objective, 1)),
                button("2").padding([5, 10])
                    .on_press(Message::ChangeTurret(ScopeTurret::Objective, 2)),
                button("3").padding([5, 10])
                    .on_press(Message::ChangeTurret(ScopeTurret::Objective, 3)),
                button("4").padding([5, 10])
                    .on_press(Message::ChangeTurret(ScopeTurret::Objective, 4)),
                button("5").padding([5, 10])
                    .on_press(Message::ChangeTurret(ScopeTurret::Objective, 5)),
                button("6").padding([5, 10])
                    .on_press(Message::ChangeTurret(ScopeTurret::Objective, 6)),
            ]
            .spacing(5)
            .padding(5)
        ]
        .align_items(Alignment::Center);

        let density_filters = column![
            pick_list(
                "pick",
                self.turret_positions.get(&ScopeTurret::DensityFilter1),
                Message::ChangeTurret(ScopeTurret::DensityFilter1, 1),
            )
        ]
        .align_items(Alignment::Center);

        let ld_size = column![
            text("Light Diaphragm Aperture").width(Length::Fill),
            row![
                text(self.ld_value),
                Slider::new(0..=250, self.ld_value, Message::LDUpdate)
            ]
        ]
        .align_items(Alignment::Center);

        let column2 = column![
            "This is a test"
        ]
        .spacing(10)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_items(Alignment::Center);

        let content = row![
            column![
                objective_turret_selection,
                Rule::horizontal(1),
                density_filters,
                Rule::horizontal(1),
                ld_size,
            ]
            .padding(20)
            .spacing(20)
            .width(Length::Fixed(250.0)),
            Rule::vertical(1),
            column2,
        ]
        .width(Length::Fill)
        .align_items(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
