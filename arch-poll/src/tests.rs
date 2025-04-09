#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    
    // Helper function to create a standard test poll
    fn create_test_poll(contract: &mut VotingContract) -> String {
        let creator_address = "wallet_creator".to_string();
        let poll_title = "Test Poll".to_string();
        let poll_description = "This is a test poll".to_string();
        let poll_options = vec!["Option A".to_string(), "Option B".to_string(), "Option C".to_string()];
        
        // Create a poll with a 10 second duration
        contract.create_poll(creator_address, poll_title, poll_description, poll_options, 10).unwrap()
    }
    
    // Helper function to create a poll with custom parameters
    fn create_custom_poll(
        contract: &mut VotingContract,
        creator_address: String,
        poll_title: String,
        poll_description: String,
        poll_options: Vec<String>,
        poll_duration_seconds: u64,
    ) -> Result<String> {
        contract.create_poll(
            creator_address,
            poll_title,
            poll_description,
            poll_options,
            poll_duration_seconds,
        )
    }
    
    #[test]
    fn test_contract_initialization() {
        let admin_address = "wallet_admin".to_string();
        let contract = VotingContract::new(admin_address.clone());
        
        assert_eq!(contract.admin_address, admin_address);
        assert_eq!(contract.active_polls.len(), 0);
    }
    
    #[test]
    fn test_create_poll_basic() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        let poll_id = create_test_poll(&mut contract);
        
        // Verify poll was created with correct attributes
        let poll = contract.get_poll(&poll_id).unwrap();
        assert_eq!(poll.poll_title, "Test Poll");
        assert_eq!(poll.poll_description, "This is a test poll");
        assert_eq!(poll.voting_options.len(), 3);
        assert_eq!(poll.participant_addresses.len(), 0);
        assert!(poll.is_active());
        assert_eq!(poll.poll_is_closed, false);
        
        // Verify initial vote counts are zero
        for option in &poll.voting_options {
            assert_eq!(*poll.vote_counts.get(option).unwrap(), 0);
        }
    }
    
    #[test]
    fn test_create_poll_invalid_options() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        let creator_address = "wallet_creator".to_string();
        
        // Test with only one option
        let result = create_custom_poll(
            &mut contract,
            creator_address.clone(),
            "Invalid Poll".to_string(),
            "This poll has too few options".to_string(),
            vec!["Option A".to_string()],
            60,
        );
        
        assert!(matches!(result, Err(VotingError::PollCreationFailed)));
        
        // Test with empty options vector
        let result = create_custom_poll(
            &mut contract,
            creator_address.clone(),
            "Invalid Poll".to_string(),
            "This poll has no options".to_string(),
            vec![],
            60,
        );
        
        assert!(matches!(result, Err(VotingError::PollCreationFailed)));
    }
    
    #[test]
    fn test_create_poll_invalid_duration() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        let creator_address = "wallet_creator".to_string();
        
        // Test with zero duration
        let result = create_custom_poll(
            &mut contract,
            creator_address.clone(),
            "Invalid Poll".to_string(),
            "This poll has zero duration".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            0,
        );
        
        assert!(matches!(result, Err(VotingError::InvalidTimeSettings)));
    }
    
    #[test]
    fn test_vote_basic() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        let poll_id = create_test_poll(&mut contract);
        
        // Cast votes
        let voter1_address = "wallet_voter1".to_string();
        let voter2_address = "wallet_voter2".to_string();
        let voter3_address = "wallet_voter3".to_string();
        
        contract.vote(&poll_id, voter1_address, "Option A").unwrap();
        contract.vote(&poll_id, voter2_address, "Option B").unwrap();
        contract.vote(&poll_id, voter3_address, "Option A").unwrap();
        
        // Verify votes were recorded correctly
        let results = contract.get_poll_results(&poll_id).unwrap();
        assert_eq!(*results.get("Option A").unwrap(), 2);
        assert_eq!(*results.get("Option B").unwrap(), 1);
        assert_eq!(*results.get("Option C").unwrap(), 0);
        
        let poll = contract.get_poll(&poll_id).unwrap();
        assert_eq!(poll.total_votes(), 3);
        assert!(poll.participant_addresses.contains(&voter1_address));
        assert!(poll.participant_addresses.contains(&voter2_address));
        assert!(poll.participant_addresses.contains(&voter3_address));
    }
    
    #[test]
    fn test_vote_nonexistent_poll() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        let voter_address = "wallet_voter".to_string();
        let result = contract.vote("nonexistent_poll_id", voter_address, "Option A");
        
        assert!(matches!(result, Err(VotingError::PollNotFound)));
    }
    
    #[test]
    fn test_double_voting_prevention() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        let poll_id = create_test_poll(&mut contract);
        
        // First vote should succeed
        let voter_address = "wallet_voter".to_string();
        contract.vote(&poll_id, voter_address.clone(), "Option A").unwrap();
        
        // Second vote should fail with AlreadyVoted error
        let result = contract.vote(&poll_id, voter_address, "Option B");
        assert!(matches!(result, Err(VotingError::AlreadyVoted)));
        
        // Verify the first vote remains and wasn't changed
        let results = contract.get_poll_results(&poll_id).unwrap();
        assert_eq!(*results.get("Option A").unwrap(), 1);
        assert_eq!(*results.get("Option B").unwrap(), 0);
    }
    
    #[test]
    fn test_vote_invalid_option() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        let poll_id = create_test_poll(&mut contract);
        
        // Vote for non-existent option
        let voter_address = "wallet_voter".to_string();
        let invalid_option = "Option D";
        let result = contract.vote(&poll_id, voter_address, invalid_option);
        
        assert!(matches!(result, Err(VotingError::InvalidOption)));
        
        // Verify no vote was recorded
        let poll = contract.get_poll(&poll_id).unwrap();
        assert_eq!(poll.total_votes(), 0);
    }
    
    #[test]
    fn test_poll_automatic_expiration() {
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
        
        // Verify poll is initially active
        let poll_active = contract.is_poll_active(&poll_id).unwrap();
        assert!(poll_active);
        
        // Sleep to allow the poll to expire
        sleep(Duration::from_secs(2));
        
        // Verify poll is no longer active before processing
        let poll_active = contract.is_poll_active(&poll_id).unwrap();
        assert!(!poll_active);
        
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
    fn test_manual_poll_closure_by_creator() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        let creator_address = "wallet_creator".to_string();
        let poll_id = contract.create_poll(
            creator_address.clone(),
            "Creator Closure Test".to_string(),
            "This poll will be closed by creator".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            60, // 60 second duration
        ).unwrap();
        
        // Verify poll is initially active
        assert!(contract.is_poll_active(&poll_id).unwrap());
        
        // Creator can close their own poll
        contract.close_poll(&poll_id, &creator_address).unwrap();
        
        // Verify poll is closed
        let poll = contract.get_poll(&poll_id).unwrap();
        assert!(poll.poll_is_closed);
        assert!(!poll.is_active());
        assert!(!contract.is_poll_active(&poll_id).unwrap());
        
        // Attempt to vote on closed poll should fail
        let voter_address = "wallet_voter".to_string();
        let result = contract.vote(&poll_id, voter_address, "Option A");
        assert!(matches!(result, Err(VotingError::PollClosed)));
    }
    
    #[test]
    fn test_manual_poll_closure_by_admin() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address.clone());
        
        let creator_address = "wallet_creator".to_string();
        let poll_id = contract.create_poll(
            creator_address,
            "Admin Closure Test".to_string(),
            "This poll will be closed by admin".to_string(),
            vec!["Yes".to_string(), "No".to_string()],
            60,
        ).unwrap();
        
        // Verify poll is initially active
        assert!(contract.is_poll_active(&poll_id).unwrap());
        
        // Admin can close any poll
        contract.close_poll(&poll_id, &admin_address).unwrap();
        
        // Verify poll is closed
        let poll = contract.get_poll(&poll_id).unwrap();
        assert!(poll.poll_is_closed);
        assert!(!poll.is_active());
    }
    
    #[test]
    fn test_unauthorized_poll_closure() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        let creator_address = "wallet_creator".to_string();
        let poll_id = contract.create_poll(
            creator_address,
            "Unauthorized Closure Test".to_string(),
            "Random users cannot close this poll".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            60,
        ).unwrap();
        
        // Random user cannot close the poll
        let random_user_address = "wallet_random".to_string();
        let result = contract.close_poll(&poll_id, &random_user_address);
        assert!(matches!(result, Err(VotingError::NotAuthorized)));
        
        // Verify poll remains active
        let poll = contract.get_poll(&poll_id).unwrap();
        assert!(!poll.poll_is_closed);
        assert!(poll.is_active());
    }
    
    #[test]
    fn test_get_nonexistent_poll() {
        let admin_address = "wallet_admin".to_string();
        let contract = VotingContract::new(admin_address);
        
        let result = contract.get_poll("nonexistent_poll_id");
        assert!(matches!(result, Err(VotingError::PollNotFound)));
        
        let result = contract.get_poll_results("nonexistent_poll_id");
        assert!(matches!(result, Err(VotingError::PollNotFound)));
        
        let result = contract.is_poll_active("nonexistent_poll_id");
        assert!(matches!(result, Err(VotingError::PollNotFound)));
    }
    
    #[test]
    fn test_multiple_polls() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        let creator_address = "wallet_creator".to_string();
        
        // Create several polls
        let poll_id1 = contract.create_poll(
            creator_address.clone(),
            "Poll 1".to_string(),
            "First test poll".to_string(),
            vec!["Yes".to_string(), "No".to_string()],
            60,
        ).unwrap();
        
        let poll_id2 = contract.create_poll(
            creator_address.clone(),
            "Poll 2".to_string(),
            "Second test poll".to_string(),
            vec!["Option X".to_string(), "Option Y".to_string(), "Option Z".to_string()],
            60,
        ).unwrap();
        
        let poll_id3 = contract.create_poll(
            creator_address.clone(),
            "Poll 3".to_string(),
            "Third test poll".to_string(),
            vec!["Approve".to_string(), "Reject".to_string()],
            60,
        ).unwrap();
        
        // Verify all polls are stored correctly
        assert_eq!(contract.active_polls.len(), 3);
        
        // Get all polls and verify count
        let all_polls = contract.get_all_polls();
        assert_eq!(all_polls.len(), 3);
        
        // Verify each poll can be accessed by ID
        assert!(contract.get_poll(&poll_id1).is_ok());
        assert!(contract.get_poll(&poll_id2).is_ok());
        assert!(contract.get_poll(&poll_id3).is_ok());
        
        // Close one poll
        contract.close_poll(&poll_id2, &creator_address).unwrap();
        
        // Verify active polls count is now 2
        let active_polls = contract.get_active_polls();
        assert_eq!(active_polls.len(), 2);
        
        // Verify all polls count is still 3
        let all_polls = contract.get_all_polls();
        assert_eq!(all_polls.len(), 3);
    }
    
    #[test]
    fn test_active_polls_filtering() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        // Create multiple polls
        let creator_address = "wallet_creator".to_string();
        let poll_id1 = contract.create_poll(
            creator_address.clone(),
            "Active Poll 1".to_string(),
            "This poll is active".to_string(),
            vec!["Yes".to_string(), "No".to_string()],
            60,
        ).unwrap();
        
        let poll_id2 = contract.create_poll(
            creator_address.clone(),
            "Soon to be Closed Poll".to_string(),
            "This poll will be closed".to_string(),
            vec!["Yes".to_string(), "No".to_string()],
            60,
        ).unwrap();
        
        let poll_id3 = contract.create_poll(
            creator_address.clone(),
            "Short-lived Poll".to_string(),
            "This poll will expire".to_string(),
            vec!["Yes".to_string(), "No".to_string()],
            1,
        ).unwrap();
        
        // Close one poll manually
        contract.close_poll(&poll_id2, &creator_address).unwrap();
        
        // Wait for short poll to expire
        sleep(Duration::from_secs(2));
        contract.process_expired_polls();
        
        // Check active polls
        let active_polls = contract.get_active_polls();
        assert_eq!(active_polls.len(), 1);
        assert_eq!(active_polls[0].poll_id, poll_id1);
        
        // All polls should still be accessible
        assert_eq!(contract.get_all_polls().len(), 3);
    }
    
    #[test]
    fn test_process_expired_polls() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        let creator_address = "wallet_creator".to_string();
        
        // Create polls with different durations
        let poll_id1 = contract.create_poll(
            creator_address.clone(),
            "Long Poll".to_string(),
            "This poll lasts a minute".to_string(),
            vec!["Yes".to_string(), "No".to_string()],
            60,
        ).unwrap();
        
        let poll_id2 = contract.create_poll(
            creator_address.clone(),
            "Very Short Poll".to_string(),
            "This poll expires in 1 second".to_string(),
            vec!["Yes".to_string(), "No".to_string()],
            1,
        ).unwrap();
        
        let poll_id3 = contract.create_poll(
            creator_address.clone(),
            "Short Poll".to_string(),
            "This poll expires in 2 seconds".to_string(),
            vec!["Yes".to_string(), "No".to_string()],
            2,
        ).unwrap();
        
        // Wait for two polls to expire
        sleep(Duration::from_secs(3));
        
        // Process expired polls
        let closed_poll_ids = contract.process_expired_polls();
        
        // Verify correct polls were closed
        assert_eq!(closed_poll_ids.len(), 2);
        assert!(closed_poll_ids.contains(&poll_id2));
        assert!(closed_poll_ids.contains(&poll_id3));
        assert!(!closed_poll_ids.contains(&poll_id1));
        
        // Verify first poll is still active
        assert!(contract.is_poll_active(&poll_id1).unwrap());
        
        // Verify expired polls are closed
        assert!(!contract.is_poll_active(&poll_id2).unwrap());
        assert!(!contract.is_poll_active(&poll_id3).unwrap());
    }
    
    #[test]
    fn test_voting_after_poll_closure() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        let creator_address = "wallet_creator".to_string();
        let voter_address = "wallet_voter".to_string();
        
        // Create a poll
        let poll_id = contract.create_poll(
            creator_address.clone(),
            "Will be closed".to_string(),
            "This poll will be closed mid-voting".to_string(),
            vec!["Option A".to_string(), "Option B".to_string()],
            60,
        ).unwrap();
        
        // Cast one vote
        contract.vote(&poll_id, voter_address, "Option A").unwrap();
        
        // Close the poll
        contract.close_poll(&poll_id, &creator_address).unwrap();
        
        // Try to vote after closure
        let new_voter_address = "wallet_voter2".to_string();
        let result = contract.vote(&poll_id, new_voter_address, "Option B");
        
        // Verify vote was rejected
        assert!(matches!(result, Err(VotingError::PollClosed)));
        
        // Verify original vote count remains unchanged
        let results = contract.get_poll_results(&poll_id).unwrap();
        assert_eq!(*results.get("Option A").unwrap(), 1);
        assert_eq!(*results.get("Option B").unwrap(), 0);
    }
    
    #[test]
    fn test_multiple_voters_same_option() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        
        let poll_id = create_test_poll(&mut contract);
        
        // Multiple voters selecting the same option
        let voter_addresses = vec![
            "wallet_voter1".to_string(),
            "wallet_voter2".to_string(),
            "wallet_voter3".to_string(),
            "wallet_voter4".to_string(),
            "wallet_voter5".to_string(),
        ];
        
        // All voting for Option A
        for voter_address in voter_addresses {
            contract.vote(&poll_id, voter_address, "Option A").unwrap();
        }
        
        // Verify vote counts
        let results = contract.get_poll_results(&poll_id).unwrap();
        assert_eq!(*results.get("Option A").unwrap(), 5);
        assert_eq!(*results.get("Option B").unwrap(), 0);
        assert_eq!(*results.get("Option C").unwrap(), 0);
        
        // Verify total vote count
        let poll = contract.get_poll(&poll_id).unwrap();
        assert_eq!(poll.total_votes(), 5);
    }
    
    #[test]
    fn test_unique_poll_ids() {
        let admin_address = "wallet_admin".to_string();
        let mut contract = VotingContract::new(admin_address);
        let creator_address = "wallet_creator".to_string();
        
        // Create multiple polls
        let poll_id1 = contract.create_poll(
            creator_address.clone(),
            "Poll 1".to_string(),
            "First poll".to_string(),
            vec!["Yes".to_string(), "No".to_string()],
            60,
        ).unwrap();
        
        let poll_id2 = contract.create_poll(
            creator_address.clone(),
            "Poll 2".to_string(),
            "Second poll".to_string(),
            vec!["Yes".to_string(), "No".to_string()],
            60,
        ).unwrap();
        
        let poll_id3 = contract.create_poll(
            creator_address.clone(),
            "Poll 3".to_string(),
            "Third poll".to_string(),
            vec!["Yes".to_string(), "No".to_string()],
            60,
        ).unwrap();
        
        // Verify all poll IDs are unique
        assert_ne!(poll_id1, poll_id2);
        assert_ne!(poll_id1, poll_id3);
        assert_ne!(poll_id2, poll_id3);
    }
}