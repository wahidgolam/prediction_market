#![feature(const_float_bits_conv)]
use anchor_lang::prelude::*;
use core::f64::consts::E;
use core::f64::consts::LN_2;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::system_instruction;

declare_id!("ChQAkMgywFmdYq1538UJc2yuLRVuskHhfceE8TBGFixx");

#[program]
mod hello_anchor {
    use super::*;

    //CONSTANTS
    const BONDING_CURVE_PARAM: u32 = 1;// * 10 ^ 3;
    const WITHDRAW_FEE: u64 = LAMPORTS_PER_SOL/(10 as u64);

    //FUNCTIONS

    
    pub fn create_market(ctx: Context<CreateMarket>, metadata: InitMarketParams) -> Result<()> {


        let INITIAL_RESERVE_LAMPORTS: u64 =
            ((BONDING_CURVE_PARAM as f64 * LN_2) * LAMPORTS_PER_SOL as f64).trunc() as u64;
        let new_market = &mut ctx.accounts.market;
        let creator = &mut ctx.accounts.signer;
        
        if new_market.initialised == true {
            return err!(ProgramError::MarketAlreadyInitialised);
        }

        //INITIAL_RESERVE_LAMPORTS TRANSFER
        let transfer_instruction =
            system_instruction::transfer(creator.key, &new_market.key(), INITIAL_RESERVE_LAMPORTS);

        anchor_lang::solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                creator.to_account_info(),
                new_market.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[],
        )?;

        //INITIALISING MARKET VARIABLES
        new_market.id = metadata.id;
        new_market.initialised = true;
        new_market.supply_y = 0;
        new_market.supply_n = 0;
        new_market.price_y = 0;
        new_market.price_n = 0;
        new_market.settled = false;

        msg!(
            "Market created with inital reserve of {} lamports",
            INITIAL_RESERVE_LAMPORTS
        );

        Ok(())
    }

    pub fn take_position(ctx: Context<TakePosition>, metadata: TakePositionParams) -> Result<()> {

        let market = &mut ctx.accounts.market;
        let position = &mut ctx.accounts.position;
        let creator = &mut ctx.accounts.signer;

        let del_reserve: u64;
        let order_size: u32 = metadata.number;
        let order_type: bool = metadata.position_type;

        position.taker = creator.key();
        position.position_type = metadata.position_type;
        position.withdrawn = false;

        let current_reserve_lamports: u64 = market.get_lamports();
        let current_supply_y: u32 = market.supply_y;
        let current_supply_n: u32 = market.supply_n;

        if order_type == true {
            let new_reserve_lamports: u64 = (BONDING_CURVE_PARAM as f64
                * ((E.powf((current_supply_y + order_size) as f64 / (BONDING_CURVE_PARAM as f64)))
                    + (E.powf((current_supply_n as f64) / (BONDING_CURVE_PARAM as f64))))
                    .ln()
                * LAMPORTS_PER_SOL as f64).trunc() as u64;

            del_reserve = new_reserve_lamports - current_reserve_lamports;
            market.supply_y += order_size;
            market.price_y =
                ((del_reserve as f64 / order_size as f64) * LAMPORTS_PER_SOL as f64).trunc() as u64;
        } else {
            let new_reserve_lamports: u64 = (BONDING_CURVE_PARAM as f64
                * ((E.powf((current_supply_y) as f64 / (BONDING_CURVE_PARAM as f64)))
                    + (E.powf((current_supply_n  + order_size) as f64 / (BONDING_CURVE_PARAM as f64))))
                    .ln()
                * LAMPORTS_PER_SOL as f64)
                .trunc() as u64;
            del_reserve = new_reserve_lamports - current_reserve_lamports;
            market.supply_n += order_size;
            market.price_n =
                ((del_reserve as f64 / order_size as f64) * LAMPORTS_PER_SOL as f64).trunc() as u64;
        }
        if del_reserve>0 {
            //TRANSFER TO MARKET
            let transfer_instruction =
            system_instruction::transfer(creator.key, &market.key(), del_reserve);

            anchor_lang::solana_program::program::invoke_signed(
                &transfer_instruction,
                &[
                    creator.to_account_info(),
                    market.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                &[],
            )?;
        }
        else{
            //TRANSFER TO USER
            let transfer_instruction =
            system_instruction::transfer(&market.key(), creator.key, del_reserve);

            anchor_lang::solana_program::program::invoke_signed(
                &transfer_instruction,
                &[
                    market.to_account_info(),
                    creator.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                &[],
            )?;
        }

        position.del_reserve += del_reserve;
        position.number += metadata.number;

        Ok(())
    }

    pub fn close_position(ctx: Context<TakePosition>, metadata: TakePositionParams) -> Result<()> {

        let market = &mut ctx.accounts.market;
        let position = &mut ctx.accounts.position;
        let creator = &mut ctx.accounts.signer;

        let del_reserve: u64;
        let order_size: u32 = metadata.number;
        let order_type: bool = metadata.position_type;
        
        if order_size>position.number {
            return err!(ProgramError::NotEnoughTokens);
        }

        position.taker = creator.key();
        position.position_type = metadata.position_type;
        position.withdrawn = false;

        let current_reserve_lamports: u64 = market.get_lamports();
        let current_supply_y: u32 = market.supply_y;
        let current_supply_n: u32 = market.supply_n;

        if order_type == true {
            let new_reserve_lamports: u64 = (BONDING_CURVE_PARAM as f64
                * ((E.powf((current_supply_y - order_size) as f64 / (BONDING_CURVE_PARAM as f64)))
                    + (E.powf((current_supply_n as f64) / (BONDING_CURVE_PARAM as f64))))
                    .ln()
                * LAMPORTS_PER_SOL as f64)
                .trunc() as u64;

            del_reserve = current_reserve_lamports - new_reserve_lamports;
            market.supply_y = market.supply_y - order_size;
            market.price_y =
                ((del_reserve as f64 / order_size as f64) * LAMPORTS_PER_SOL as f64).trunc() as u64;
        } else {
            let new_reserve_lamports: u64 = (BONDING_CURVE_PARAM as f64
                * ((E.powf((current_supply_y) as f64 / (BONDING_CURVE_PARAM as f64)))
                    + (E.powf((current_supply_n  - order_size) as f64 / (BONDING_CURVE_PARAM as f64))))
                    .ln()
                * LAMPORTS_PER_SOL as f64)
                .trunc() as u64;
            del_reserve = current_reserve_lamports - new_reserve_lamports;
            market.supply_n = market.supply_y - order_size;
            market.price_n =
                ((del_reserve as f64 / order_size as f64) * LAMPORTS_PER_SOL as f64).trunc() as u64;
        }
        let transfer_instruction =
            system_instruction::transfer(&market.key(), creator.key, del_reserve);

        anchor_lang::solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                market.to_account_info(),
                creator.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[],
        )?;

        position.del_reserve = position.del_reserve - del_reserve;
        position.number = position.number - metadata.number;

        Ok(())
    }

    pub fn settle_market(ctx: Context<SettleMarket>, metadata: SettleMarketParams) -> Result<()> {
        let market = &mut ctx.accounts.market;
        if market.settled{
            return err!(ProgramError::MarketAlreadySettled);
        }
        market.settled = true;
        market.result = metadata.result;

        Ok(())
    }

    pub fn claim_amount(ctx: Context<WithdrawPosition>, _metadata: WithdrawPositionParams) -> Result<()> {

        let INITIAL_RESERVE_LAMPORTS: u64 =
            ((BONDING_CURVE_PARAM as f64 * LN_2) * LAMPORTS_PER_SOL as f64).trunc() as u64;
        
        let market = &mut ctx.accounts.market;
        let position = &mut ctx.accounts.position;
        let creator = &mut ctx.accounts.signer;
        let position_type:bool = position.position_type;
        let number: u32 = position.number;
        let current_reserve_lamports = market.get_lamports();

        //CHECK: Market has settled
        if market.settled!=true {
            return err!(ProgramError::MarketNotSettled);
        }
        if position.withdrawn==true {
            return err!(ProgramError::PositionAlreadyWithdrawn);
        }

        if market.result==position_type{
            //WIN
            let supply: u32;
            
            if market.result {
                supply = market.supply_y
            }
            else{
                supply = market.supply_n;
            }
            let transfer_amount = ((number as f64/supply as f64) * (current_reserve_lamports as f64 - INITIAL_RESERVE_LAMPORTS as f64)).trunc() as u64 - WITHDRAW_FEE;
            position.withdrawn = true;

            let transfer_instruction =
            system_instruction::transfer(&market.key(), creator.key, transfer_amount);

            anchor_lang::solana_program::program::invoke_signed(
                &transfer_instruction,
                &[  
                    market.to_account_info(),
                    creator.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                &[],
            )?;
        }
        else{
            return err!(ProgramError::MarketSettledAgainstPosition);
        }
        Ok(())
    }
}

#[account]
pub struct Market {
    initialised: bool,//1
    id: String,       //40
    supply_y: u32,    //4
    supply_n: u32,    //4 
    settled: bool,    //1
    result: bool,     //1
    price_y: u64,     //8
    price_n: u64,     //8
} // 67 + 8 = 75 bytes

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitMarketParams {
    pub id: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct SettleMarketParams {
    pub id: String,
    pub result: bool,
}

#[derive(Accounts)]
#[instruction(metadata: InitMarketParams)]
pub struct CreateMarket<'info> {
    #[account(
        init_if_needed,
        payer = signer,
        space = 75,
        seeds = [metadata.id.as_bytes().as_ref()],
        bump
    )]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(metadata: SettleMarketParams)]
pub struct SettleMarket<'info> {
    #[account(
        mut,
        seeds = [metadata.id.as_bytes().as_ref()],
        bump
    )]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Position {
    taker: Pubkey,       //32
    position_type: bool, //1
    number: u32,         //8
    del_reserve: u64,    //8
    withdrawn: bool      //1
} //50 + 8 = 58 bytes

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct TakePositionParams {
    pub market_id: String,
    pub position_type_string: String,
    pub position_type: bool,
    pub number: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct WithdrawPositionParams {
    pub market_id: String,
    pub position_type_string: String,
    pub position_type: bool,
}

#[derive(Accounts)]
#[instruction(metadata: TakePositionParams)]
pub struct TakePosition<'info> {
    #[account(
        mut,
        seeds = [metadata.market_id.as_bytes().as_ref()],
        bump
    )]
    pub market: Account<'info, Market>,
    #[account(
        init_if_needed,
        payer = signer,
        space = 58,
        seeds = [metadata.market_id.as_bytes().as_ref(), signer.key().as_ref(), metadata.position_type_string.as_bytes().as_ref()],
        bump
    )]
    pub position: Account<'info, Position>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(metadata: WithdrawPositionParams)]
pub struct WithdrawPosition<'info> {
    #[account(
        mut,
        seeds = [metadata.market_id.as_bytes().as_ref()],
        bump
    )]
    pub market: Account<'info, Market>,
    #[account(
        mut,
        seeds = [metadata.market_id.as_bytes().as_ref(), signer.key().as_ref(), metadata.position_type_string.as_bytes().as_ref()],
        bump
    )]
    pub position: Account<'info, Position>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum ProgramError {
    #[msg("Market can not be created, already initialised")]
    MarketAlreadyInitialised,
    #[msg("Position can not be sold: not enough tokens")]
    NotEnoughTokens,
    #[msg("Market can not be settled: already settled")]
    MarketAlreadySettled,
    #[msg("Position can not be withdrawn: market not settled")]
    MarketNotSettled,
    #[msg("Position can not be withdrawn: position already withdrawn")]
    PositionAlreadyWithdrawn,
    #[msg("Position can not be withdrawn: market settled against position")]
    MarketSettledAgainstPosition
}