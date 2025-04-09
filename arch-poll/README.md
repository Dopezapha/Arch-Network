# Voting Contract Documentation

## Overview

The Voting Contract is a Rust-based smart contract that enables secure and transparent voting. It allows users to create polls, cast votes, track results, and manage poll lifetimes with automatic or manual closure mechanisms.

## Core Modules

### Error Handling

```rust
pub enum VotingError {
    PollNotFound,        // Returned when trying to access a non-existent poll
    PollClosed,          // Returned when attempting an action on a closed poll
    AlreadyVoted,        // Returned when a user tries to vote twice
    NotAuthorized,       // Returned when permission is denied for an action
    InvalidOption,       // Returned when voting for a non-existent option
    PollCreationFailed,  // Returned when poll creation parameters are invalid
    InvalidTimeSettings, // Returned when poll duration settings are invalid
}

pub type Result<T> = std::result::Result<T, VotingError>;
```

### Poll Structure

The `Poll` struct encapsulates all data and functionality related to individual polls:

```rust
pub struct Poll {
    pub poll_id: String,                      // Unique identifier for the poll
    pub poll_title: String,                   // Title of the poll
    pub poll_description: String,             // Description explaining the poll
    pub voting_options: Vec<String>,          // Available voting options
    pub vote_counts: HashMap<String, usize>,  // Maps options to vote counts
    pub participant_addresses: HashSet<String>, // Set of addresses that have voted
    pub poll_creator_address: String,         // Address of the poll creator
    pub poll_start_timestamp: u64,            // Unix timestamp when poll starts
    pub poll_end_timestamp: u64,              // Unix timestamp when poll ends
    pub poll_is_closed: bool,                 // Whether the poll is closed
}
```

#### Poll Methods

```rust
impl Poll {
    // Checks if the poll is currently active (not closed and within time bounds)
    pub fn is_active(&self) -> bool;
    
    // Returns the current voting results for all options
    pub fn get_results(&self) -> HashMap<String, usize>;
    
    // Returns the total number of votes cast in the poll
    pub fn total_votes(&self) -> usize;
    
    // Marks the poll as closed
    pub fn close(&mut self);
}
```

### VotingContract Structure

The main contract that manages all polls and voting operations:

```rust
pub struct VotingContract {
    pub active_polls: HashMap<String, Poll>, // Maps poll IDs to Poll objects
    pub admin_address: String,               // Address of the contract admin
}
```

## Core Functions

### Initialization

```rust
// Creates a new voting contract with the specified admin address
pub fn new(admin_address: String) -> Self;
```

### Poll Management

```rust
// Creates a new poll with the specified parameters
pub fn create_poll(
    &mut self,
    creator_address: String,     // Address of poll creator
    poll_title: String,          // Title of the poll
    poll_description: String,    // Description of the poll
    poll_options: Vec<String>,   // Available voting options
    poll_duration_seconds: u64,  // Duration in seconds
) -> Result<String>;             // Returns poll ID if successful

// Manually closes a poll (admin or creator only)
pub fn close_poll(
    &mut self, 
    poll_id: &str,           // ID of the poll to close
    wallet_address: &str     // Address of user requesting closure
) -> Result<()>;

// Automatically checks and closes polls that have passed their end time
pub fn process_expired_polls(&mut self) -> Vec<String>;  // Returns IDs of closed polls
```

### Voting Operations

```rust
// Cast a vote in a poll
pub fn vote(
    &mut self,
    poll_id: &str,           // ID of the poll to vote in
    voter_address: String,   // Address of the voter
    selected_option: &str    // Option selected by the voter
) -> Result<()>;
```

### Query Functions

```rust
// Gets details of a specific poll
pub fn get_poll(&self, poll_id: &str) -> Result<&Poll>;

// Gets results of a specific poll
pub fn get_poll_results(&self, poll_id: &str) -> Result<HashMap<String, usize>>;

// Checks if a poll is currently active
pub fn is_poll_active(&self, poll_id: &str) -> Result<bool>;

// Gets all polls in the contract
pub fn get_all_polls(&self) -> Vec<&Poll>;

// Gets only active polls
pub fn get_active_polls(&self) -> Vec<&Poll>;
```

## Usage Examples

### Creating a New Poll

```rust
let admin_address = "admin_wallet_address".to_string();
let mut contract = VotingContract::new(admin_address);

let creator_address = "creator_wallet_address".to_string();
let poll_title = "Community Decision".to_string();
let poll_description = "Should we implement feature X?".to_string();
let poll_options = vec!["Yes".to_string(), "No".to_string(), "Abstain".to_string()];
let poll_duration = 60 * 60 * 24 * 7; // 7 days in seconds

let poll_id = match contract.create_poll(
    creator_address,
    poll_title,
    poll_description,
    poll_options,
    poll_duration
) {
    Ok(id) => id,
    Err(e) => panic!("Failed to create poll: {:?}", e),
};
```

### Casting a Vote

```rust
let voter_address = "voter_wallet_address".to_string();
let selected_option = "Yes";

match contract.vote(&poll_id, voter_address, selected_option) {
    Ok(_) => println!("Vote successfully cast!"),
    Err(VotingError::AlreadyVoted) => println!("You have already voted in this poll"),
    Err(VotingError::PollClosed) => println!("This poll is closed"),
    Err(e) => println!("Error casting vote: {:?}", e),
};
```

### Getting Poll Results

```rust
match contract.get_poll_results(&poll_id) {
    Ok(results) => {
        println!("Current poll results:");
        for (option, count) in results {
            println!("{}: {} votes", option, count);
        }
    },
    Err(e) => println!("Error getting results: {:?}", e),
};
```

### Managing Poll Lifecycle

```rust
// Process any expired polls
let closed_polls = contract.process_expired_polls();
if !closed_polls.is_empty() {
    println!("The following polls were closed due to expiration:");
    for poll_id in closed_polls {
        println!("- {}", poll_id);
    }
}

// Manually close a poll (as admin or creator)
let admin_address = "admin_wallet_address".to_string();
match contract.close_poll(&poll_id, &admin_address) {
    Ok(_) => println!("Poll successfully closed"),
    Err(e) => println!("Error closing poll: {:?}", e),
};
```

## Security Considerations

1. **Double-Voting Prevention**: The contract tracks all voter addresses in a HashSet to prevent users from voting multiple times.

2. **Access Control**: Only the admin or poll creator can manually close a poll, preventing unauthorized manipulation.

3. **Time-Bounded Polls**: Polls automatically close after their duration expires, enforcing time-bound voting periods.

4. **Input Validation**: The contract validates all inputs, including poll options, vote selections, and time settings.

5. **Error Handling**: Comprehensive error types ensure clear feedback when operations fail.

## Performance Considerations

1. **Efficient Data Structures**: HashMaps and HashSets provide O(1) lookups for voter verification and vote counting.

2. **Minimal Storage**: Only essential data is stored to minimize blockchain storage costs.

3. **Batch Processing**: The `process_expired_polls` function allows for efficient batch closure of multiple expired polls.