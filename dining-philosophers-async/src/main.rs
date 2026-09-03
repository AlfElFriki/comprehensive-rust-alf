use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::time;

struct Chopstick;

struct Philosopher {
	name: String,
	left_chopstick: Arc<Mutex<Chopstick>>,
	right_chopstick: Arc<Mutex<Chopstick>>,
	thoughts: mpsc::Sender<String>,
}

impl Philosopher {
	async fn think(&self) {
		self.thoughts
			.send(format!("Eureka! {} has a new idea!", &self.name))
			.await
			.unwrap();
	}

	async fn eat(&self) {
		// Keep trying until we have both chopsticks
		let left = self.left_chopstick.lock().await;
		let right = self.right_chopstick.lock().await;
		println!("{} is eating...", &self.name);
		time::sleep(time::Duration::from_millis(5)).await;
		drop(left);
		drop(right);
	}
}

// tokio scheduler doesn't deadlock with 5 philosophers, so have 2.
static PHILOSOPHERS: &[&str] = &["Socrates", "Hypatia"];

#[tokio::main]
async fn main() {
	// Create chopsticks
	let chopsticks: Vec<Arc<Mutex<Chopstick>>> = (0..PHILOSOPHERS.len())
		.map(|_| Arc::new(Mutex::new(Chopstick)))
		.collect();
	// Create philosophers
	let (thoughts_tx, mut thoughts_rx) = mpsc::channel(100);
	let philosophers: Vec<Philosopher> = PHILOSOPHERS
		.iter()
		.enumerate()
		.map(|(i, &name)| Philosopher {
			name: name.to_string(),
			left_chopstick: chopsticks[i].clone(),
			right_chopstick: chopsticks[(i + 1) % PHILOSOPHERS.len()].clone(),
			thoughts: thoughts_tx.clone(),
		})
		.collect();
	// Make them think and eat
	for philosopher in philosophers {
		let p = philosopher;
		tokio::spawn(async move {
			for _ in 0..5 {
				p.think().await;
				p.eat().await;
			}
		});
	}
	// Output their thoughts
	while let Some(thought) = thoughts_rx.recv().await {
		println!("{}", thought);
	}
}
