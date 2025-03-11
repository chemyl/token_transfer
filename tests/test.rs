use token_transfer::processor::process_instruction;
use {
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        pubkey::Pubkey,
        rent::Rent,
        system_instruction,
    },
    solana_program_test::{ProgramTest, processor, tokio},
    solana_sdk::{signature::Signer, signer::keypair::Keypair, transaction::Transaction},
    spl_token::state::{Account, Mint},
    std::str::FromStr,
};

#[tokio::test]
async fn success() {
    // setup some pubkeys for the accounts
    let program_id = Pubkey::from_str("TransferTokens11111111111111111111111111111").unwrap();
    let source = Keypair::new();
    let mint = Keypair::new();
    let destination = Keypair::new();

    // retrive PDA Account
    let (authority_pubkey, _) = Pubkey::find_program_address(&[b"authority"], &program_id);
    let rent = Rent::default();

    // token info
    let amount = 10_000;
    let decimals = 9;

    // start test env with 'process_instruction'
    let program_test = ProgramTest::new(
        "spl_example_transfer_tokens",
        program_id,
        processor!(process_instruction),
    );

    // start test blockchain
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Create and set up Token Account
    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &mint.pubkey(),
                rent.minimum_balance(Mint::LEN),
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                &payer.pubkey(),
                None,
                decimals,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &mint],
        recent_blockhash,
    );
    // run transaction on test blockchain
    banks_client.process_transaction(transaction).await.unwrap();

    // Create Source Account and connect to the Mint and PDA
    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &source.pubkey(),
                rent.minimum_balance(Account::LEN),
                Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &source.pubkey(),
                &mint.pubkey(),
                &authority_pubkey,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &source],
        recent_blockhash,
    );
    // run transaction on test blockchain
    banks_client.process_transaction(transaction).await.unwrap();

    // Create Destination Account and connect to the Mint
    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &destination.pubkey(),
                rent.minimum_balance(Account::LEN),
                Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &destination.pubkey(),
                &mint.pubkey(),
                &payer.pubkey(),
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &destination],
        recent_blockhash,
    );
    // run transaction on test blockchain
    banks_client.process_transaction(transaction).await.unwrap();

    // Mint some tokens to the source Account
    let transaction = Transaction::new_signed_with_payer(
        &[spl_token::instruction::mint_to(
            &spl_token::id(),
            &mint.pubkey(),
            &source.pubkey(),
            &payer.pubkey(),
            &[],
            amount,
        )
        .unwrap()],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    // run transaction on test blockchain
    banks_client.process_transaction(transaction).await.unwrap();

    let transfer_amount: u64 = 5000;
    let instruction_data = transfer_amount.to_le_bytes(); // 8 байт LE

    // Create an function that invoke 'process_instruction'
    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &(instruction_data),
            vec![
                AccountMeta::new(source.pubkey(), false),           // from
                AccountMeta::new_readonly(mint.pubkey(), false),    // token
                AccountMeta::new(destination.pubkey(), false),      // destination
                AccountMeta::new_readonly(authority_pubkey, false), // PDA for authorization
                AccountMeta::new_readonly(spl_token::id(), false),  // SLP programm
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    // process the transfer transaction
    banks_client.process_transaction(transaction).await.unwrap();

    // Check that the destination account now has `amount` tokens
    let account = banks_client
        .get_account(destination.pubkey())
        .await
        .unwrap()
        .unwrap();
    // get destination account
    let token_account = Account::unpack(&account.data).unwrap();
    assert_eq!(token_account.amount, transfer_amount);
}
