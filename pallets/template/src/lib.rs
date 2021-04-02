#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure, StorageMap, dispatch, traits::Get};
use frame_system::ensure_signed;

use sp_std::vec::Vec; 

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Config> as TemplateModule {
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		Something get(fn something): Option<u32>;

		Tasks: map hasher(blake2_128_concat) u128 => (T::AccountId, u8, u128, Vec<u8>, T::BlockNumber);
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, AccountId),

		/// 当任务被创建时触发此事件 字段：[who, task_id, task_status, task_reward, task_detail]
        TaskCreated(AccountId, u128, u8, u128, Vec<u8>),
        /// 当任务被更新 字段：[who, task_id, task_status, task_reward, task_detail]
        TaskUpdated(AccountId, u128, u8, u128, Vec<u8>),
        /// 当任务被撤销时触发此事件 字段：[who, task_id]
		TaskRevoked(AccountId, u128),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		/// 该工作任务已经存在
		TaskAlreadyExisted,
		/// 该工作任务不存在
		NoSuchTask,
		/// 该工作任务不是本人创建的，因此它不能被撤销
		NotTaskOwner,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			// Update storage.
			Something::put(something);

			// Emit an event.
			Self::deposit_event(RawEvent::SomethingStored(something, who));
			// Return a successful DispatchResult
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn cause_error(origin) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match Something::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					Something::put(new);
					Ok(())
				},
			}
		}

		/// 创建任务
        #[weight = 10_000]
        fn create_task(origin, task_id: u128, task_status: u8, task_reward: u128, task_detail: Vec<u8>) -> dispatch::DispatchResult {
            // 检查当前操作者是否已签名
            // 如果未签名，则返回错误
            let publisher = ensure_signed(origin)?;

            // 检查任务ID是否已存在
            ensure!(!Tasks::<T>::contains_key(&task_id), Error::<T>::TaskAlreadyExisted);

            // 从FRAME系统模块中获取当前区块高度
            let current_block = <frame_system::Module<T>>::block_number();

            // 将任务数据存储到链上
            Tasks::<T>::insert(&task_id, (&publisher, task_status.clone(), task_reward.clone(), task_detail.clone(), current_block));

            // 触发创建任务事件 
			Self::deposit_event(RawEvent::TaskCreated(publisher, task_id, task_status, task_reward, task_detail));

			// Return a successful DispatchResult
			Ok(())
			
        }

        /// 更新任务
        #[weight = 10_000]
        fn update_task(origin, task_id: u128, task_status: u8, task_reward: u128, task_detail: Vec<u8>) -> dispatch::DispatchResult {
            // 检查调用者是否已签名
            // 如果未签名，则函数将返回错误
            let publisher = ensure_signed(origin)?;

            // 检查任务是否存在
            ensure!(Tasks::<T>::contains_key(&task_id), Error::<T>::NoSuchTask);

            // 获取任务所有者
            let (owner, _, _, _, _) = Tasks::<T>::get(&task_id);

            // 检查调用者是否为任务的所有者
            ensure!(publisher == owner, Error::<T>::NotTaskOwner);

            // 从FRAME系统模块获取块号
            let current_block = <frame_system::Module<T>>::block_number();

            // 修改任务
            Tasks::<T>::insert(&task_id, (&publisher, task_status.clone(), task_reward.clone(), task_detail.clone(), current_block));

            // 触发修改任务事件
            Self::deposit_event(RawEvent::TaskUpdated(publisher, task_id, task_status, task_reward, task_detail));

			// Return a successful DispatchResult
			Ok(())

        }

        /// 撤销任务
        #[weight = 10_000]
        fn revoke_task(origin, task_id: u128) -> dispatch::DispatchResult {
            // 检查当前操作者是否已签名
            // 如果未签名，则返回错误
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let publisher = ensure_signed(origin)?;

            // 检查任务是否存在
            ensure!(Tasks::<T>::contains_key(&task_id), Error::<T>::NoSuchTask);

            // 获取任务所有者
            let (owner, _, _, _, _) = Tasks::<T>::get(&task_id);

            // 检查当前操作者是否为任务的所有者
            ensure!(publisher == owner, Error::<T>::NotTaskOwner);

            // 从链上存储中撤销任务
            Tasks::<T>::remove(&task_id);

            // 触发撤销任务事件
            Self::deposit_event(RawEvent::TaskRevoked(publisher, task_id));

			// Return a successful DispatchResult
			Ok(())

		}

		
	}
}
