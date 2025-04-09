// This is a voting contract written in Rust.
// This enables users to create polls, vote on options, and view results transparently.
// The contract is tested with unit tests to ensure its functionality and reliability.
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

// Define the error types for our contract
#[derive(Debug)]
pub enum VotingError {
    PollNotFound,
    PollClosed,
    AlreadyVoted,
    NotAuthorized,
    InvalidOption,
    PollCreationFailed,
    InvalidTimeSettings,
}

// Define the result type for our contract functions
pub type Result<T> = std::result::Result<T, VotingError>;

// Define the Poll structure
#[derive(Debug, Clone)]
pub struct Poll {
    pub poll_id: String,
    pub poll_title: String,
    pub poll_description: String,
    pub voting_options: Vec<String>,
    pub vote_counts: HashMap<String, usize>, // Maps options to vote counts
    pub participant_addresses: HashSet<String>, // Set of wallet addresses that have voted
    pub poll_creator_address: String,       // Wallet address of creator
    pub poll_start_timestamp: u64,          // Unix timestamp
    pub poll_end_timestamp: u64,            // Unix timestamp
    pub poll_is_closed: bool,               // Whether the poll is closed
}

impl Poll {
    // Check if the poll is currently active
    pub fn is_active(&self) -> bool {
        if self.poll_is_closed {
            return false;
        }
        
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        current_timestamp >= self.poll_start_timestamp && current_timestamp < self.poll_end_timestamp
    }
    
    // Get the current results of the poll
    pub fn get_results(&self) -> HashMap<String, usize> {
        self.vote_counts.clone()
    }
    
    // Get the total number of votes cast
    pub fn total_votes(&self) -> usize {
        self.participant_addresses.len()
    }
    
    // Close the poll
    pub fn close(&mut self) {
        self.poll_is_closed = true;
    }
}

// Define the voting contract
pub struct VotingContract {
    pub active_polls: HashMap<String, Poll>,
    pub admin_address: String, // The admin wallet address
}

impl VotingContract {
    // Create a new voting contract
    pub fn new(admin_address: String) -> Self {
        VotingContract {
            active_polls: HashMap::new(),
            admin_address,
        }
    }
    
    // Create a new poll
    pub fn create_poll(
        &mut self,
        creator_address: String,
        poll_title: String,
        poll_description: String,
        poll_options: Vec<String>,
        poll_duration_seconds: u64,
    ) -> Result<String> {
        // Basic validation
        if poll_options.len() < 2 {
            return Err(VotingError::PollCreationFailed);
        }
        
        // Generate unique ID for the poll
        let poll_id = format!("poll_{}", self.active_polls.len() + 1);
        
        // Set up time boundaries
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        if poll_duration_seconds == 0 {
            return Err(VotingError::InvalidTimeSettings);
        }
        
        let poll_end_timestamp = current_timestamp + poll_duration_seconds;
        
        // Initialize vote counts for each option
        let mut option_vote_counts = HashMap::new();
        for voting_option in &poll_options {
            option_vote_counts.insert(voting_option.clone(), 0);
        }
        
        // Create and store the poll
        let new_poll = Poll {
            poll_id: poll_id.clone(),
            poll_title,
            poll_description,
            voting_options: poll_options,
            vote_counts: option_vote_counts,
            participant_addresses: HashSet::new(),
            poll_creator_address: creator_address,
            poll_start_timestamp: current_timestamp,
            poll_end_timestamp,
            poll_is_closed: false,
        };
        
        self.active_polls.insert(poll_id.clone(), new_poll);
        
        Ok(poll_id)
    }
    
    // Cast a vote in a poll
    pub fn vote(&mut self, poll_id: &str, voter_address: String, selected_option: &str) -> Result<()> {
        // Retrieve poll or return error
        let poll = self.active_polls.get_mut(poll_id).ok_or(VotingError::PollNotFound)?;
        
        // Check if poll is active
        if !poll.is_active() {
            return Err(VotingError::PollClosed);
        }
        
        // Check if voter has already voted
        if poll.participant_addresses.contains(&voter_address) {
            return Err(VotingError::AlreadyVoted);
        }
        
        // Check if option is valid
        if !poll.voting_options.contains(&selected_option.to_string()) {
            return Err(VotingError::InvalidOption);
        }
        
        // Record the vote
        let option_count = poll.vote_counts.entry(selected_option.to_string()).or_insert(0);
        *option_count += 1;
        
        // Record that this wallet has voted
        poll.participant_addresses.insert(voter_address);
        
        Ok(())
    }
    
    // Get details of a specific poll
    pub fn get_poll(&self, poll_id: &str) -> Result<&Poll> {
        self.active_polls.get(poll_id).ok_or(VotingError::PollNotFound)
    }
    
    // Get results of a specific poll
    pub fn get_poll_results(&self, poll_id: &str) -> Result<HashMap<String, usize>> {
        let poll = self.get_poll(poll_id)?;
        Ok(poll.get_results())
    }
    
    // Check if the poll is active
    pub fn is_poll_active(&self, poll_id: &str) -> Result<bool> {
        let poll = self.get_poll(poll_id)?;
        Ok(poll.is_active())
    }
    
    // Get all polls
    pub fn get_all_polls(&self) -> Vec<&Poll> {
        self.active_polls.values().collect()
    }
    
    // Get all active polls
    pub fn get_active_polls(&self) -> Vec<&Poll> {
        self.active_polls.values().filter(|poll| poll.is_active()).collect()
    }
    
    // Manually close a poll (admin or creator only)
    pub fn close_poll(&mut self, poll_id: &str, wallet_address: &str) -> Result<()> {
        let poll = self.active_polls.get_mut(poll_id).ok_or(VotingError::PollNotFound)?;
        
        // Only admin or poll creator can close the poll
        if wallet_address != &self.admin_address && wallet_address != &poll.poll_creator_address {
            return Err(VotingError::NotAuthorized);
        }
        
        poll.close();
        Ok(())
    }
    
    // Automatically check and close polls that have passed their end time
    pub fn process_expired_polls(&mut self) -> Vec<String> {
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let mut closed_poll_ids = Vec::new();
        
        for (poll_id, poll) in self.active_polls.iter_mut() {
            if !poll.poll_is_closed && current_timestamp >= poll.poll_end_timestamp {
                poll.close();
                closed_poll_ids.push(poll_id.clone());
            }
        }
        
        closed_poll_ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;
    
    // Helper function to create a test poll
    fn create_test_poll(contract: &mut VotingContract) -> String {
        let creator_address = "wallet_creator".to_string();
        let poll_title = "Test Poll".to_string();
        let poll_description = "This is a test poll".to_string();
        let poll_options = vec!["Option A".to_string(), "Option B".to_string(), "Option C".to_string()];
        
        // Create a poll with a 10 second duration
        contract.create_poll(creator_address, poll_title, poll_description, poll_options, 10).unwrap()
    }
    
    #[test]
    fn test_create_poll() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        let poll_id = create_test_poll(&mut contract);
        
        // Verify poll was created
        let poll = contract.get_poll(&poll_id).unwrap();
        assert_eq!(poll.poll_title, "Test Poll");
        assert_eq!(poll.voting_options.len(), 3);
        assert_eq!(poll.participant_addresses.len(), 0);
        assert!(poll.is_active());
    }
    
    #[test]
    fn test_vote() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        let poll_id = create_test_poll(&mut contract);
        
        // Cast votes
        let voter1_address = "wallet_voter1".to_string();
        let voter2_address = "wallet_voter2".to_string();
        
        contract.vote(&poll_id, voter1_address, "Option A").unwrap();
        contract.vote(&poll_id, voter2_address, "Option B").unwrap();
        
        // Verify votes were recorded
        let results = contract.get_poll_results(&poll_id).unwrap();
        assert_eq!(*results.get("Option A").unwrap(), 1);
        assert_eq!(*results.get("Option B").unwrap(), 1);
        assert_eq!(*results.get("Option C").unwrap(), 0);
        
        let poll = contract.get_poll(&poll_id).unwrap();
        assert_eq!(poll.total_votes(), 2);
    }
    
    #[test]
    fn test_double_voting_prevention() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        let poll_id = create_test_poll(&mut contract);
        
        // First vote should succeed
        let voter_address = "wallet_voter".to_string();
        contract.vote(&poll_id, voter_address.clone(), "Option A").unwrap();
        
        // Second vote should fail
        let result = contract.vote(&poll_id, voter_address, "Option B");
        assert!(matches!(result, Err(VotingError::AlreadyVoted)));
    }
    
    #[test]
    fn test_invalid_option() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        let poll_id = create_test_poll(&mut contract);
        
        // Vote for non-existent option
        let voter_address = "wallet_voter".to_string();
        let result = contract.vote(&poll_id, voter_address, "Option D");
        assert!(matches!(result, Err(VotingError::InvalidOption)));
    }
    
    #[test]
    fn test_poll_expiration() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        // Create a poll with a very short duration for testing
        let creator_address = "wallet_creator".to_string();
        let poll_id = contract.create_poll(
            creator_address,
            "Short Poll".to_string(),
            "This poll expires quickly".to_string(),
            vec!["Yes".to_string(), "No".to_string()],
            1, // 1 second duration
        ).unwrap();
        
        // Sleep to allow the poll to expire
        sleep(Duration::from_secs(2));
        
        // Process expired polls
        let closed_poll_ids = contract.process_expired_polls();
        assert!(closed_poll_ids.contains(&poll_id));
        
        // Verify the poll is now closed
        let poll = contract.get_poll(&poll_id).unwrap();
        assert!(poll.poll_is_closed);
        
        // Attempt to vote on expired poll should fail
        let voter_address = "wallet_voter".to_string();
        let result = contract.vote(&poll_id, voter_address, "Yes");
        assert!(matches!(result, Err(VotingError::PollClosed)));
    }
    
    #[test]
    fn test_manual_poll_closure() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address.clone());
        
        let creator_address = "wallet_creator".to_string();
        let poll_id = contract.create_poll(
            creator_address.clone(),
            "Test Poll".to_string(),
            "This is a test poll".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            60, // 60 second duration
        ).unwrap();
        
        // Creator can close their own poll
        contract.close_poll(&poll_id, &creator_address).unwrap();
        
        // Verify poll is closed
        let poll = contract.get_poll(&poll_id).unwrap();
        assert!(poll.poll_is_closed);
        
        // Create another poll for admin closure test
        let poll_id2 = contract.create_poll(
            creator_address,
            "Admin Test Poll".to_string(),
            "This poll will be closed by admin".to_string(),
            vec!["Yes".to_string(), "No".to_string()],
            60,
        ).unwrap();
        
        // Admin can close any poll
        contract.close_poll(&poll_id2, &admin_address).unwrap();
        
        // Verify poll is closed
        let poll = contract.get_poll(&poll_id2).unwrap();
        assert!(poll.poll_is_closed);
    }
    
    #[test]
    fn test_unauthorized_poll_closure() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        let creator_address = "wallet_creator".to_string();
        let poll_id = contract.create_poll(
            creator_address,
            "Test Poll".to_string(),
            "This is a test poll".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            60,
        ).unwrap();
        
        // Random user cannot close the poll
        let random_user_address = "wallet_random".to_string();
        let result = contract.close_poll(&poll_id, &random_user_address);
        assert!(matches!(result, Err(VotingError::NotAuthorized)));
    }
    
    #[test]
    fn test_active_polls_filtering() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        // Create two polls
        let creator_address = "wallet_creator".to_string();
        let poll_id1 = contract.create_poll(
            creator_address.clone(),
            "Active Poll".to_string(),
            "This poll is active".to_string(),
            vec!["Yes".to_string(), "No".to_string()],
            60,
        ).unwrap();
        
        let poll_id2 = contract.create_poll(
            creator_address.clone(),
            "Closed Poll".to_string(),
            "This poll will be closed".to_string(),
            vec!["Yes".to_string(), "No".to_string()],
            60,
        ).unwrap();
        
        // Close one poll
        contract.close_poll(&poll_id2, &creator_address).unwrap();
        
        // Check active polls
        let active_polls = contract.get_active_polls();
        assert_eq!(active_polls.len(), 1);
        assert_eq!(active_polls[0].poll_id, poll_id1);
    }
}