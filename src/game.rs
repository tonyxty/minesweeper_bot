use telegram_bot::*;

pub type Coord = (usize, usize);

pub struct InteractResult {
    pub update_text: Option<String>,
    pub update_board: Option<InlineKeyboardMarkup>,
    pub game_end: bool,
}

pub trait Game {
    fn interact(&mut self, coord: Coord, user: &User) -> Option<InteractResult>;
}


impl InteractResult {
    pub async fn reply_to(self, api: &Api, message: &Message) -> Result<(), Error> {
        if let Some(text) = self.update_text {
            if let Some(board) = self.update_board {
                api.send(message.edit_text(text).reply_markup(board)).await.map(|_| ())
            } else {
                api.send(message.edit_text(text)).await.map(|_| ())
            }
        } else if let Some(board) = self.update_board {
            api.send(message.edit_reply_markup(Some(board))).await.map(|_| ())
        } else {
            Ok(())
        }
    }
}
