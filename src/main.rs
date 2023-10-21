#![recursion_limit = "1024"]

use zeiss_control::{Scope, ScopeTurret};
use vgtk::{ext::*, gtk, run, Component, UpdateAction, VNode};
use vgtk::lib::{gtk::*, gio::ApplicationFlags};

use vgtk::lib::gdk_pixbuf::{Pixbuf, Colorspace};
use vgtk::lib::glib::Bytes;

use nokhwa::CallbackCamera;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::pixel_format::RgbFormat;

#[derive(Clone, Default)]
struct Model {
    ld_value: u8,
    scope: Option<Scope>,
}

#[derive(Clone, Debug)]
enum Message {
   SendData,
   SetCounter(u8),
   Exit,
}

impl Component for Model {
    type Message = Message;
    type Properties = ();

    fn update(&mut self, message: Message) -> UpdateAction<Self> {
        match message {
            Message::SendData => {
                println!("Hi!");
                UpdateAction::Render
            }
            Message::SetCounter(value) => {
                self.ld_value = value;
                UpdateAction::Render
            }
            Message::Exit => {
                vgtk::quit();
                UpdateAction::None
            }
        }
    }

    fn view(&self) -> VNode<Model> {
        if self.scope.is_none() {
            let scope = match Scope::new(
                "/dev/ttyUSB1",
                "/dev/ttyUSB0"
            ) {
                Ok(scope) => Some(scope),
                Err(error) => {
                    None
                }
            };
        }

        let index = CameraIndex::Index(0);
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
        let mut camera = CallbackCamera::new(index, requested, |_| {}).unwrap();
        camera.open_stream().unwrap();

        // get a frame
        let frame = camera.poll_frame().unwrap();
        let decoded = frame.decode_image::<RgbFormat>().unwrap();
        let img_width = decoded.width();
        let img_height = decoded.height();
        let buf = Pixbuf::from_mut_slice(
            decoded.into_raw(),
            Colorspace::Rgb,
            false,
            8,
            img_width.try_into().unwrap(),
            img_height.try_into().unwrap(),
            img_width.try_into().unwrap(),
        );
        let buf = buf.scale_simple(
            640,
            480,
            vgtk::lib::gdk_pixbuf::InterpType::Hyper
        );

        gtk! {
            <Application::new_unwrap(None, ApplicationFlags::empty())>
                <Window border_width=20 on destroy=|_| Message::Exit>
                    <HeaderBar title="Zeiss Controller" show_close_button=true />
                    <Box spacing=10>
                        <Box spacing=10 halign=Align::Start valign=Align::Start orientation=Orientation::Vertical>
                            <Box spacing=10>
                                <Label label="Diaphragm: " />

                                <ScaleButton image="go-next" always_show_image=true
                                    on value_changed=|_, val| {
                                        let new_value = (255.0 * (val / 100.0)) as u8;
                                        Message::SetCounter(new_value)
                                    }
                                    on popdown=|_| Message::SendData />

                                <Label label={
                                    let mut value = ((self.ld_value as f32 / 2.55) as u8).to_string();
                                    value.push_str("%");
                                    value
                                } />
                            </Box>
                            <@TurretButtons turret=Some(ScopeTurret::Objective) scope=self.scope.clone() />
                        </Box>
                        <Box>
                            <Image />
                        </Box>
                    </Box>
                </Window>
            </Application>
        }
    }
}

fn main() {
   std::process::exit(run::<Model>());
}

#[derive(Clone, Debug, Default)]
pub struct TurretButtons {
    turret: Option<ScopeTurret>,
    scope: Option<Scope>
}

#[derive(Clone, Debug)]
pub enum TurretSelectMessage {
    Switch(u8)
}

impl Component for TurretButtons {
    type Message = TurretSelectMessage;
    type Properties = Self;

    fn create(props: Self) -> Self {
        props
    }

    fn change(&mut self, props: Self) -> UpdateAction<Self> {
        *self = props;
        UpdateAction::Render
    }

    fn update(&mut self, msg: Self::Message) -> UpdateAction<Self> {
        match msg {
            TurretSelectMessage::Switch(position) => {
                println!("Setting {:?} to {}", self.turret.unwrap_or(ScopeTurret::Unknown), position);
                match &self.scope {
                    Some(scope) => scope.clone().set_turret_pos(self.turret.unwrap_or(ScopeTurret::Unknown), position).unwrap(),
                    None => ()
                }
            }
        }
        UpdateAction::None
    }

    fn view(&self, ) -> VNode<Self> {
        gtk! {
            <Box spacing=10 orientation=Orientation::Vertical halign=Align::Start>
                <Label label={format!("{:?}", self.turret.unwrap_or(ScopeTurret::Unknown))} halign=Align::Start />
                <Box spacing=10 halign=Align::Center>
                {
                    (1..=self.turret.unwrap_or(ScopeTurret::Unknown).positions()).map(|counter| {
                        gtk! {
                            <Box hexpand=true >
                                <Button label=counter.to_string() on clicked=|_| TurretSelectMessage::Switch(counter) />
                            </Box>
                        }
                    })
                }
                </Box>
            </Box>
        }
    }
}
