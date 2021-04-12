// This file is part of Acala.

// Copyright (C) 2020-2021 Acala Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement::KeepAlive},
	transactional,
};
use frame_system::pallet_prelude::*;
use orml_traits::{BasicCurrency, BasicReservableCurrency, NFT};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{AccountIdConversion, StaticLookup, Zero},
	DispatchResult, ModuleId, RuntimeDebug,
};
use sp_std::vec::Vec;

pub mod weights;
pub use weights::WeightInfo;
pub use module::*;

pub type Balance = u128;
pub type Metadata = sp_std::vec::Vec<u8>;
// pub type Name = sp_std::vec::Vec<u8>;
// pub type Description = sp_std::vec::Vec<u8>;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ClassType {
	/// Token can be transferred
	Transferable,
	/// Token can be burned
	Burnable,
}

impl ClassType {
	pub fn is_transferable(&self) -> bool{
		match *self{
			ClassType::Transferable => true,
			_ => false,
		}
	}
}
impl Default for ClassType {
	fn default() -> Self {
        ClassType::Transferable
    }
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ClassData {
	/// The minimum balance to create class
	pub deposit: Balance,
	/// Property of token
	pub classtype: ClassType,
	/// Name of class.
	pub name: Vec<u8>,
	/// Description of class.
	pub description: Vec<u8>,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TokenData {
	/// The minimum balance to create token
	pub deposit: Balance,
}

pub type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
pub type TokenInfoOf<T> = orml_nft::TokenInfo<<T as frame_system::Config>::AccountId, <T as orml_nft::Config>::TokenData>;


#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config + orml_nft::Config<ClassData = ClassData, TokenData = TokenData> + pallet_proxy::Config
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The minimum balance to create class
		#[pallet::constant]
		type CreateClassDeposit: Get<Balance>;

		/// The minimum balance to create token
		#[pallet::constant]
		type CreateTokenDeposit: Get<Balance>;

		/// The NFT's module id
		#[pallet::constant]
		type ModuleId: Get<ModuleId>;

		///  Currency type for reserve/unreserve balance to
		/// create_class/mint/burn/destroy_class
		type Currency: BasicReservableCurrency<Self::AccountId, Balance = Balance>;
		// type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	/// 查询在一个class下的账户有多少nft
	#[pallet::storage]
	#[pallet::getter(fn owned_tokens_count)]
	pub type OwnedTokensCount<T: Config> = 
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, T::ClassId, u32, ValueQuery>;


	#[pallet::error]
	pub enum Error<T> {
		/// ClassId not found
		ClassIdNotFound,
		/// TokenId not found
		TokenIdNotFound,
		/// The operator is not the owner of the token and has no permission
		NoPermission,
		/// Quantity is invalid. need >= 1
		InvalidQuantity,
		/// Property of class don't support transfer
		NonTransferable,
		/// Property of class don't support burn
		NonBurnable,
		/// Can not destroy class
		/// Total issuance is not 0
		CannotDestroyClass,
	}
	
	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Created NFT class. \[owner, class_id\]
		CreatedClass(T::AccountId, ClassIdOf<T>),
		/// Minted NFT token. \[from, to, class_id, token_id\]
		MintedToken(T::AccountId, T::AccountId, ClassIdOf<T>, TokenIdOf<T>),
		/// Transferred NFT token. \[from, to, class_id, token_id\]
		TransferredToken(T::AccountId, T::AccountId, ClassIdOf<T>, TokenIdOf<T>),
		/// Burned NFT token. \[owner, class_id, token_id\]
		BurnedToken(T::AccountId, ClassIdOf<T>, TokenIdOf<T>),
		/// Destroyed NFT class. \[owner, class_id, dest\]
		DestroyedClass(T::AccountId, ClassIdOf<T>, T::AccountId),
		/// query balance
        Balance(self::Balance),
        /// query owner
        Owner(Option<T::AccountId>),
        /// query token metadata
        TokensInfo(TokenInfoOf<T>),
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create NFT class, tokens belong to the class.
		///
		/// - `metadata`: external metadata
		/// - `properties`: class property, include `Transferable` `Burnable`
		#[pallet::weight(<T as Config>::WeightInfo::create_class())]
		#[transactional]
		pub fn create_class(origin: OriginFor<T>, metadata: Metadata, classtype: ClassType, name: Vec<u8>, description: Vec<u8>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let next_id = orml_nft::Pallet::<T>::next_class_id();
			let owner: T::AccountId = T::ModuleId::get().into_sub_account(next_id);
			let deposit = T::CreateClassDeposit::get();

			// it depends https://github.com/paritytech/substrate/issues/7563
			<T as Config>::Currency::transfer(&who, &owner, deposit)?;
			// Currently, use `free_balance(owner)` instead of `deposit`.
			<T as Config>::Currency::reserve(&owner, <T as Config>::Currency::free_balance(&owner))?;

			// owner add proxy delegate to origin
			let proxy_deposit = <pallet_proxy::Module<T>>::deposit(1u32);
			<T as pallet_proxy::Config>::Currency::transfer(&who, &owner, proxy_deposit, KeepAlive)?;
			<pallet_proxy::Module<T>>::add_proxy_delegate(&owner, who, Default::default(), Zero::zero())?;

			let data = ClassData { deposit, classtype, name, description};
			orml_nft::Pallet::<T>::create_class(&owner, metadata, data)?;

			Self::deposit_event(Event::CreatedClass(owner, next_id));
			Ok(().into())
		}

		/// 创建NFT
		///
		/// - `to`: the token owner's account
		/// - `class_id`: token belong to the class id
		/// - `metadata`: external metadata
		/// - `reward_amount`: 任务奖励锁定
		#[pallet::weight(<T as Config>::WeightInfo::mint())]
		#[transactional]
		pub fn mint(
			origin: OriginFor<T>,
			to: <T::Lookup as StaticLookup>::Source,
			class_id: ClassIdOf<T>,
			metadata: Metadata,
			reward_amount: u32,
			// quantity: u32,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			ensure!(reward_amount >= 1, Error::<T>::InvalidQuantity);
			let class_info = orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
			ensure!(who == class_info.owner, Error::<T>::NoPermission);
			let deposit = T::CreateTokenDeposit::get();
			let total_deposit = deposit * (reward_amount as u128);
			// <T as Config>::Currency::reserve(&to, total_deposit)?;
			// 质押数量到合约
			<T as Config>::Currency::transfer(&to, &who, total_deposit)?;
			<T as Config>::Currency::reserve(&who, total_deposit)?;

			let data = TokenData { deposit:total_deposit, };
			
			let token_id = orml_nft::Pallet::<T>::mint(&to, class_id, metadata.clone(), data.clone())?;
		
			Self::deposit_event(Event::MintedToken(who, to, class_id, token_id));
			Ok(().into())
		}

		/// 任务奖励发放
		#[pallet::weight(<T as Config>::WeightInfo::transfer())]
		#[transactional]
		pub fn recieve_task(
			origin: OriginFor<T>,
			token: (ClassIdOf<T>, TokenIdOf<T>)
		) -> DispatchResultWithPostInfo{
			
			Ok(().into())
		}

		/// Transfer NFT token to another account
		///
		/// - `to`: the token owner's account
		/// - `token`: (class_id, token_id)
		#[pallet::weight(<T as Config>::WeightInfo::transfer())]
		#[transactional]
		pub fn transfer(
			origin: OriginFor<T>,
			to: <T::Lookup as StaticLookup>::Source,
			token: (ClassIdOf<T>, TokenIdOf<T>),
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			Self::do_transfer(&who, &to, token)?;
			Ok(().into())
		}

		/// 任务撤销
		///
		/// - `token`: (class_id, token_id)
		#[pallet::weight(<T as Config>::WeightInfo::burn())]
		#[transactional]
		pub fn burn(origin: OriginFor<T>, token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let class_info = orml_nft::Pallet::<T>::classes(token.0).ok_or(Error::<T>::ClassIdNotFound)?;
			let data = class_info.data;
			ensure!(
				data.classtype == ClassType::Burnable,
				 Error::<T>::NonBurnable
			);
			let token_info = orml_nft::Pallet::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenIdNotFound)?;
			ensure!(who == token_info.owner, Error::<T>::NoPermission);

			orml_nft::Pallet::<T>::burn(&who, token)?;
			let owner: T::AccountId = T::ModuleId::get().into_sub_account(token.0);
			let data = token_info.data;
			// `repatriate_reserved` will check `to` account exist and return `DeadAccount`.
			// `transfer` not do this check.
			<T as Config>::Currency::unreserve(&owner, data.deposit);
			<T as Config>::Currency::transfer(&owner, &who, data.deposit)?;
			// <T as Config>::Currency::unreserve(&who, data.deposit);

			Self::deposit_event(Event::BurnedToken(who, token.0, token.1));
			Ok(().into())
		}

		/// Destroy NFT class
		///
		/// - `class_id`: destroy class id
		/// - `dest`: transfer reserve balance from sub_account to dest
		#[pallet::weight(<T as Config>::WeightInfo::destroy_class())]
		#[transactional]
		pub fn destroy_class(
			origin: OriginFor<T>,
			class_id: ClassIdOf<T>,
			dest: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(dest)?;
			let class_info = orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
			ensure!(who == class_info.owner, Error::<T>::NoPermission);
			ensure!(
				class_info.total_issuance == Zero::zero(),
				Error::<T>::CannotDestroyClass
			);

			let owner: T::AccountId = T::ModuleId::get().into_sub_account(class_id);
			let data = class_info.data;
			// `repatriate_reserved` will check `to` account exist and return `DeadAccount`.
			// `transfer` not do this check.
			<T as Config>::Currency::unreserve(&owner, data.deposit);
			<T as Config>::Currency::transfer(&owner, &dest, data.deposit)?;

			// transfer all free from origin to dest
			orml_nft::Pallet::<T>::destroy_class(&who, class_id)?;

			Self::deposit_event(Event::DestroyedClass(who, class_id, dest));
			Ok(().into())
		}

	}
}

impl<T: Config> Pallet<T> {
	/// Ensured atomic.
	#[transactional]
	fn do_transfer(from: &T::AccountId, to: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
		let class_info = orml_nft::Pallet::<T>::classes(token.0).ok_or(Error::<T>::ClassIdNotFound)?;
		let data = class_info.data;
		ensure!(
			data.classtype == ClassType::Transferable,
			Error::<T>::NonTransferable
		);

		let token_info = orml_nft::Pallet::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenIdNotFound)?;
		ensure!(*from == token_info.owner, Error::<T>::NoPermission);

		orml_nft::Pallet::<T>::transfer(from, to, token)?;

		Self::deposit_event(Event::TransferredToken(from.clone(), to.clone(), token.0, token.1));
		Ok(())
	}
}

pub type NFTBalance = u128;
impl<T: Config> NFT<T::AccountId> for Pallet<T> {
	type ClassId = ClassIdOf<T>;
	type TokenId = TokenIdOf<T>;
	type Balance = NFTBalance;

	fn balance(who: &T::AccountId) -> Self::Balance {
		orml_nft::TokensByOwner::<T>::iter_prefix(who).count() as u128
	}

	fn owner(token: (Self::ClassId, Self::TokenId)) -> Option<T::AccountId> {
		orml_nft::Pallet::<T>::tokens(token.0, token.1).map(|t| t.owner)
	}

	fn transfer(from: &T::AccountId, to: &T::AccountId, token: (Self::ClassId, Self::TokenId)) -> DispatchResult {
		Self::do_transfer(from, to, token)
	}
}