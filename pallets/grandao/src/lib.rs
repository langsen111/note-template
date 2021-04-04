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
	trait Store for Module<T: Config> as GrandaoModule {
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		Tasks: map hasher(blake2_128_concat) u128 => (T::AccountId, u8, u128, Vec<u8>, T::BlockNumber);
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
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
