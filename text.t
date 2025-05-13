Level 1
#[derive(Debug)]
struct Player {
    name: String,
    score: u32,
}

fn main() {
    let name: String = String::from("Thread");
    let score: u32 = 0;
    let player: Player = Player { name, score };
    println!("Player: {:#?}", player);
    let player2: Player = player;
    println!("Player: {:#?}", player2);
    // println!("{:#?}", player); player has been moved after it value was moved to player2
}

Level 2
#[derive(Debug)]
struct Quest {
    name: String,
    reward: u32,
    completed: bool
}
impl Quest{
    fn new(name: &str, reward: u32) -> Quest{
        Quest { name: String::from(name), reward, completed: false }
    }

    fn get_name(&self) -> &str{
        &self.name
    }

    fn complete(&mut self){
        self.reward += 10;
        self.completed = true;
    }
}

fn main(){
    let name: &str = "Math Challenge";
    let reward: u32 = 50; 
    let mut quest1: Quest = Quest::new(name, reward);
    println!("Initial: {:#?}", quest1);
    println!("Quest Name: {}", quest1.get_name());
    quest1.complete();
    println!("Updated: {:#?}", quest1);
}

Level 3

#[derive(Debug)]
struct Player{
    name: String,
    score: u32
}

#[derive(Debug)]
struct Game {
    player: Player,
    level: u32,
    active: bool
}

impl Player{
    fn new(name: &str) -> Self{
        Player { name: String::from(name), score: 0 }
    }

    fn add_score(&mut self, points: u32){
        self.score += points;
    }
}

impl Game{
    fn new(player_name: &str) -> Self{
        let player: Player = Player::new(player_name);
        Game { player, level: 1, active: true }
    }

    fn play(&mut self) -> &str{
        if self.active{
            self.level += 1;
            self.player.score += 100;
        }
        if self.level > 3{
            self.active = false;
            return "Game Over"
        }
        "Level Up!"
    }

    fn get_player_name(&self) -> &str{
        &self.player.name
    }
}

fn main() {
    let player_name: &str = "Thread";
    let mut game: Game = Game::new(player_name);
    println!("Initial: {:#?}", game);
    println!("Play 1: {}", game.play());
    println!("Play 2: {}", game.play());
    println!("Player name: {}", game.get_player_name());
    println!("Final: {:#?}", game);
}