// First we include what we are going to need in our program.
// This  is the Rust style of importing things.
// Remember we added the dependencies in cargo.toml
// And from the `solana_program` crate we are including  all the required things.

use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

// Every solana program has one entry point
// And it is a convention to name it `process_instruction`. 
// It should take in program_id, accounts, instruction_data as parameters.

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // We check if We have a instruction_data len greater than 0 if it is not we do not want to procced.
    // So we return Error with InvalidInstructionData message.
     if instruction_data.len() == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }
    // Now we just check and call the function for each of them.
    // I have choosen 0 for create_campaign,
    // 1 for withdraw
    // 2 for donate.
    if instruction_data[0] == 0 {
        return create_campaign(
            program_id,
            accounts,
            // Notice we pass program_id and accounts as they where 
            // but we pass a reference to slice of [instruction_data]. 
            // we do not want the first element in any of our functions.
            &instruction_data[1..instruction_data.len()],
        );
    } else if instruction_data[0] == 1 {
        return withdraw(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
        );
    } else if instruction_data[0] == 2 {
        return donate(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
        );
    }

    // If instruction_data doesn't match we give an error.
    // Note I have used msg!() macro and passed a string here. 
    // It is good to do this as this would 
    // also get printed in the console window
    // if a program fails.
    msg!("Didn't find the entrypoint required");
    Err(ProgramError::InvalidInstructionData)
}

// Then we call the entry point macro to add `process_instruction` as our entry point to our program.

entrypoint!(process_instruction);

//functions

#[derive(BorshSerialize, BorshDeserialize, Debug)]
    struct CampaignDetails {
        pub admin: Pubkey,
        pub name: String,
        pub description: String,
        pub image_link: String,
        pub amount_donated: u64,
    }

fn create_campaign(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {

    // We create a iterator on accounts
    // accounts parameter is the array of accounts related to this entrypoint
    let accounts_iter = &mut accounts.iter();

    //Solana programs can only write data on a program-owned account. Note that writing_account is a program-owned account.
    // Writing account or we can call it program account.
    // This is an account we will create in our front-end.
    // This account should br owned by the solana program.

    //This function returns a result enum. 
    //We can use ? Operator on the result enum to get the value.
    let writing_account = next_account_info(accounts_iter)?;


    // Account of the person creating the campaign.
    let creator_account = next_account_info(accounts_iter)?;

    // Now to allow transactions we want the creator account to sign the transaction.
    if !creator_account.is_signer {
        msg!("the creator should be signer");
        return Err(ProgramError::IncorrectProgramId);
    }

    // We want to write in this account so we want its owner by the program.
    if writing_account.owner != program_id {
        msg!("writing account isn't owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }


    //the double colon (::) is the path separator. Paths are comprised of crates, modules, and items.

    /*
        By deriving the trait BorshDeserialize in our CampaignDetails struct we have added a method try_from_slice which takes in the parameter array of u8 and creates an object of CampaignDetails with it. It gives us an enum of type results. We will use the expect method on result enums to and pass in the string which we can see in case of error.
    */

    let mut input_data = CampaignDetails::try_from_slice(&instruction_data)
        .expect("Instruction data serialization didn't worked");

    //for a campaign created the only admin should be the one who created it.
    //we can add additonal logic

    if input_data.admin != *creator_account.key {
        msg!("Invalid instruction data");
        return Err(ProgramError::InvalidInstructionData);
    }

    // get the minimum balance we need in our program account.
    let rent_exemption = Rent::get()?.minimum_balance(writing_account.data_len());

    // And we make sure our program account (`writing_account`) has that much lamports(balance).
    if **writing_account.lamports.borrow() < rent_exemption {
        msg!("The balance of writing account should be more than rent exemption");
        return Err(ProgramError::InsufficientFunds);
    }

    // Then we can set the initial amount donate to be zero.
    input_data.amount_donated = 0;

    /*
        If all goes well, we will write the writing_account. Here on our input_data variable (of type CampaignDetails), we have a method serialize. this is because of the BorshSerialize derivation. We will use this to write the data in a writing_account. At the end of the program, we can return Ok(()).
    */

    input_data.serialize(&mut &mut writing_account.data.borrow_mut()[..])?;

    Ok(())

}

fn withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {

    #[derive(BorshSerialize, BorshDeserialize, Debug)]
    struct WithdrawRequest {
        pub amount: u64,
    }

    //For the withdraw also we will create iterator and get writing_account (which is the program owned account) and admin_account.
    let accounts_iter = &mut accounts.iter();
    let writing_account = next_account_info(accounts_iter)?;
    let admin_account = next_account_info(accounts_iter)?;

    // We check if the writing account is owned by program.
    if writing_account.owner != program_id {
        msg!("writing account is not owned by the owner");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Admin account should be the signer in this trasaction.
    if !admin_account.is_signer {
        msg!("The aadmin should be a signer");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Now we will get the data of campaign from the writing_account. Note that we stored this when we created the campaign with create_campaign function.
    //input_data.serialize(&mut &mut writing_account.data.borrow_mut()[..])?;

    //Note: We are currently thinking of a scenario where a campaign is created and we want to withdraw some money from it.

    // Just like we used the try_from_slice for 
    // instruction_data we will use it for the 
    // writing_account's data.

    let campaign_data = CampaignDetails::try_from_slice(*writing_account.data.borrow())
        .expect("Error deserializing data");

    // Then we check if the admin_account's public key is equal to
    // the public key we have stored in our campaign_data.
    if campaign_data.admin != *admin_account.key {
        msg!("Only the account admin can withdraw");
        return Err(ProgramError::InvalidAccountData)
    }

    // Here we make use of the struct we created.
    // We will get the amount of lamports admin wants to withdraw
    let input_data = WithdrawRequest::try_from_slice(&instruction_data)
        .expect("Instruction data serialization didn't worked");

    // We do not want the campaign to get deleted after a withdrawal. We want it to always have a minimum balance, So we calculate the rent_exemption and consider it.
    let rent_exemption = Rent::get()?.minimum_balance(writing_account.data_len());

    // We check if we have enough funds
    if **writing_account.lamports.borrow() - rent_exemption < input_data.amount {
        msg!("Insufficent balance");
        return Err(ProgramError::InsufficientFunds);
    }

    // Transfer balance
    // We will decrease the balance of the program account, and increase the admin_account balance.

    **writing_account.try_borrow_mut_lamports()? -= input_data.amount;
    **admin_account.try_borrow_mut_lamports()? += input_data.amount;

    Ok(())
}

fn donate(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {

    let accounts_iter = &mut accounts.iter();
    let writing_account = next_account_info(accounts_iter)?;
    let donator_program_account = next_account_info(accounts_iter)?;
    let donator = next_account_info(accounts_iter)?;

    // We get 3 accounts here, first is the program-owned account containing the data of campaign we want to donate to. Then we have a donator_program_account which is also the program-owned account that only has the Lamport we would like to donate. Then we have the account of the donator.

    //When we will create it in our front-end, although we do not want to store anything in it we will assign it some size. That is so it gets automatically deleted after the SOL token has been transferred. Then we want the donator account to sign this transaction.

    if writing_account.owner != program_id {
        msg!("writing account isn't owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }

    if donator_program_account.owner != program_id {
        msg!("donator program account isn't owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }

    if !donator.is_signer {
        msg!("donator should be signer");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Here we get the campaign_data and we will increment the amount_donated, as the total amount of data donated to this campaign will increase.

    let mut campaign_data = CampaignDetails::try_from_slice(*writing_account.data.borrow()).expect("Error deserializing data");

    campaign_data.amount_donated += **donator_program_account.lamports.borrow();

    // Then we do the actual transaction. Note that the donator_program_account is owned by program so it can decrease its Lamports.

    **writing_account.try_borrow_mut_lamports()? += **donator_program_account.lamports.borrow();

    **donator_program_account.try_borrow_mut_lamports()? = 0;

    campaign_data.serialize(&mut &mut writing_account.data.borrow_mut()[..])?;

    Ok(())
}



