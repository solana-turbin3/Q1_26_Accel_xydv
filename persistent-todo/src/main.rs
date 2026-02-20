use std::{
    collections::VecDeque,
    fs::File,
    io::{Read, Write},
    time::{SystemTime, UNIX_EPOCH},
};

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, PartialEq, BorshSerialize, BorshDeserialize)]
struct Todo {
    id: u64,
    description: String,
    created_at: u64,
}

pub struct Queue<T> {
    items: VecDeque<T>,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Self {
            items: VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, item: T) {
        self.items.push_back(item);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.items.pop_front()
    }

    pub fn peek(&self) -> Option<&T> {
        self.items.front()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn items(&self) -> &VecDeque<T> {
        &self.items
    }
}

struct PersistentTodo<T> {
    queue: Queue<T>,
}

impl<T> PersistentTodo<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn save(&self) -> std::io::Result<()> {
        let todos: Vec<&T> = self.queue.items().iter().collect();
        let serialized_data = borsh::to_vec(&todos)?;
        let mut file = File::create("todos.bin")?;
        file.write_all(&serialized_data)
    }

    fn load() -> Self {
        let mut queue = Queue::new();

        if let Ok(mut file) = File::open("todos.bin") {
            let mut buffer = Vec::new();
            if file.read_to_end(&mut buffer).is_ok() {
                if let Ok(items) = borsh::from_slice::<Vec<T>>(&buffer) {
                    for item in items {
                        queue.enqueue(item);
                    }
                }
            }
        }

        Self { queue }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut todo: PersistentTodo<Todo> = PersistentTodo::load();

    match args[1].as_str() {
        "add" => {
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
            let desc = args.get(2).expect("missing description");

            todo.queue.enqueue(Todo {
                id: (todo.queue.len() + 1) as u64,
                description: desc.clone(),
                created_at: now.as_secs(),
            });

            todo.save()?;

            println!("added: {}", desc);
        }
        "list" => {
            for todo in todo.queue.items() {
                println!("[{}] {}", todo.id, todo.description);
            }
        }
        "done" => {
            if let Some(completed_todo) = todo.queue.dequeue() {
                todo.save()?;
                println!("completed: {}", completed_todo.description);
            } else {
                println!("no todos");
            }
        }
        _ => println!("command not found"),
    }
    Ok(())
}
