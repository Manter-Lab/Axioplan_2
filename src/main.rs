use iced::executor;
use iced::widget::{Slider, text, button, Rule, row, column, container};
use iced::window;
use iced::subscription::Subscription;
use iced::{Alignment, Application, Command, Element, Length, Settings, Theme};
use zeiss_control::{Scope, ScopeTurret};

fn main() {
    ScopeApp::run(Settings::default()).unwrap();
}

struct ScopeApp {
    scope: Option<Scope>,
    objective_position: u8,
    ld_value: u8,

}

#[derive(Debug, Clone, Copy)]
enum Message {
    Exit,
    ChangeTurret(ScopeTurret, u8),
    LDUpdate(u8),
}

impl Application for ScopeApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let scope = match Scope::new(
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

        (Self {
            scope,
            objective_position: 1,
            ld_value: 250,
        }, Command::none())
    }

    fn title(&self) -> String {
        String::from("AxioVision ∞")
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Exit => window::close(),
            Message::ChangeTurret(turret, position) => {
                self.scope.as_mut().unwrap().set_turret_pos(turret, position).unwrap();
                self.objective_position = position;
                Command::none()
            },
            Message::LDUpdate(value) => {
                self.scope.as_mut().unwrap().set_ld_pos(value).unwrap();
                self.ld_value = value;
                Command::none()
            },
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
            .padding(10)
        ]
        .padding(10)
        .align_items(Alignment::Center);

        let ld_size = column![
            text("Light Diaphragm Aperture").width(Length::Fill),
            Slider::new(0..=250, self.ld_value, Message::LDUpdate)
        ]
        .padding(10)
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
                ld_size,
            ].width(Length::Fixed(250.0)),
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
