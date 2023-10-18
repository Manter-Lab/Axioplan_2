use iced::executor;
use iced::widget::{button, column, container};
use iced::window;
use iced::{Alignment, Application, Command, Element, Length, Settings, Theme};
use zeiss_control::{Scope, ScopeTurret};

fn main() {
    Exit::run(Settings::default()).unwrap();

    let mut scope = match Scope::new(
        "/dev/ttyUSB1",
        "/dev/ttyUSB0"
    ) {
        Ok(newscope) => newscope,
        Err(errormessage) => {
            println!("ERROR: {}", errormessage);
            println!("Exiting");
            std::process::exit(0);
        }
    };

    scope.set_turret_pos(ScopeTurret::Objective, 1).unwrap();

    loop {
        println!("{}: {}: {}",
            scope.turret_pos(ScopeTurret::Objective).unwrap(),
            scope.ld_pos().unwrap(),
            scope.focus_dist().unwrap()
        );
    }
}


#[derive(Default)]
struct Exit {
    show_confirm: bool,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Confirm,
    Exit,
}

impl Application for Exit {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("AxioVision ∞")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Confirm => window::close(),
            Message::Exit => {
                self.show_confirm = true;

                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content = if self.show_confirm {
            column![
                "Are you sure you want to エィト?",
                button("Yes, exit now")
                    .padding([10, 20])
                    .on_press(Message::Confirm),
            ]
        } else {
            column![
                "Click the button to exit",
                button("Exit").padding([10, 20]).on_press(Message::Exit),
            ]
        }
        .spacing(10)
        .align_items(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}
