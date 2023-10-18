use std::collections::HashMap;

use iced::executor;
use iced::widget::{Slider, pick_list, text, text_input, button, Rule, row, column, container};
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

        let mut turret_positions = HashMap::new();
        match scope {
            Some(_) =>
                turret_positions = HashMap::from(
                    [(ScopeTurret::Objective,
                    scope.as_mut().unwrap().turret_pos(ScopeTurret::Objective).unwrap_or_default()),
                    (ScopeTurret::DensityFilter1,
                    scope.as_mut().unwrap().turret_pos(ScopeTurret::DensityFilter1).unwrap_or_default()),
                    (ScopeTurret::DensityFilter2,
                    scope.as_mut().unwrap().turret_pos(ScopeTurret::DensityFilter2).unwrap_or_default())]
                ),
            None => ()
        }

        (Self {
            scope,
            ld_value: 251,
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
                match self.scope {
                    Some(_) => {
                        self.ld_value = self.scope.as_mut().unwrap().ld_pos().unwrap();
                        self.turret_positions.insert(
                            ScopeTurret::DensityFilter1,
                            self.scope.as_mut().unwrap().turret_pos(ScopeTurret::DensityFilter1).unwrap()
                        );
                        self.turret_positions.insert(
                            ScopeTurret::DensityFilter2,
                            self.scope.as_mut().unwrap().turret_pos(ScopeTurret::DensityFilter2).unwrap()
                        );
                    },
                    None => ()
                }
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
            text("Density Filters").width(Length::Fill),
            row![
                text("Filter 1"),
                pick_list(
                    [1u8, 2u8, 3u8, 4u8].as_slice(),
                    self.turret_positions.get(&ScopeTurret::DensityFilter1).copied(),
                    |selection| {Message::ChangeTurret(ScopeTurret::DensityFilter1, selection)},
                )
            ].spacing(10).padding(2).align_items(Alignment::Center),
            row![
                text("Filter 2"),
                pick_list(
                    [1u8, 2u8, 3u8, 4u8].as_slice(),
                    self.turret_positions.get(&ScopeTurret::DensityFilter2).copied(),
                    |selection| {Message::ChangeTurret(ScopeTurret::DensityFilter2, selection)},
                )
            ].spacing(10).padding(2).align_items(Alignment::Center),
        ]
        .align_items(Alignment::Center);

        let ld_size = column![
            text("Light Diaphragm Aperture").width(Length::Fill),
            row![
                text_input(
                    &(((self.ld_value as f32 / 251.0) * 100.0) as u8).to_string(),
                    &(((self.ld_value as f32 / 251.0) * 100.0) as u8).to_string()
                ).on_input(|value| {
                    let new_value = match value.parse::<f32>() {
                        Ok(v) => ((v / 100.0) * 251.0) as u8,
                        Err(_) => 251
                    };
                    Message::LDUpdate(new_value)
                }).width(50),
                Slider::new(0..=251, self.ld_value, Message::LDUpdate)
            ].spacing(10).padding(2).align_items(Alignment::Center),
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
            .width(Length::Fixed(250.0))
            .height(Length::Fill)
            .align_items(Alignment::Start),
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
