#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure, dispatch, StorageMap, StorageDoubleMap };
use frame_system::ensure_signed;

use sp_std::vec::Vec; 
use sp_std::collections::btree_set::BTreeSet;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod config;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

// 任务状态
// 任务状态的更新默认只有任务创建者才有权限
// 但其中Delivered只有任务中标者有权限操作，Arbitrating、Judging两个状态双方都有权限操作
pub enum TaskStatus {
    Bidding     = 1, //待认领（投标中）
    Doing       = 2, //进行中
    UnDone      = 3, //已撤销    
    Delivered   = 4, //已交付
    Accepted    = 5, //已验收
    Arbitrating = 6, //仲裁中
    Judging     = 7, //审判中
    Finished    = 8, //已结束
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Config> as GrandaoModule {

        /*  任务基本信息============================================================================ */
        /// 任务列表 (TaskId, ...)
        pub TaskList get(fn task_list): BTreeSet<u128>;
        /// 任务详情 TaskId => (AccountId, TaskDetailJson, BlockNumber)
		pub TaskDetail get(fn task_detail): map hasher(twox_64_concat) u128 => (T::AccountId, Vec<u8>, T::BlockNumber);
        /// 任务总数 
        pub TaskCount get (fn task_count): u128;

        /*  任务关系信息============================================================================ */
        /// 任务与状态关系 TaskId => Status
        pub RelTaskStatus get(fn rel_task_status): map hasher(twox_64_concat) u128 => u8;
        /// 创建任务与质押的Token数量关系 TaskId => Token
        pub RelCreateTaskStake get(fn rel_create_task_stake): map hasher(twox_64_concat) u128 => u128;
        /// 投标任务与质押的Token数量关系 (TaskId, AccountId) => Token
        pub RelBidTaskStake get(fn rel_bid_stake): double_map hasher(twox_64_concat) u128, hasher(blake2_128_concat) T::AccountId => u128;
        /// 任务与投标人关系 一对多 TaskId => (AccountId, ...)
        pub RelTaskBidder get(fn rel_task_bidder): map hasher(twox_64_concat) u128 => BTreeSet<T::AccountId>;
        /// 任务与中标人关系 一对一 TaskId => AccountId
        pub RelTaskReceiver get(fn rel_task_receiver): map hasher(twox_64_concat) u128 => T::AccountId;

        /*  用户关系信息============================================================================ */   
        /// 会员列表 (AccountId, ...)
        pub UserList get(fn user_list): BTreeSet<T::AccountId>;
        /// 我创建的任务列表 AccountId => (TaskId, ...)
        pub MyCreateTasks get (fn my_create_tasks): map hasher(blake2_128_concat) T::AccountId => BTreeSet<u128>;
        /// 我投标的任务列表 AccountId => (TaskId, ...)
        pub MyBidTasks get (fn my_bid_tasks): map hasher(blake2_128_concat) T::AccountId => BTreeSet<u128>;
        /// 我中标的任务列表 AccountId => (TaskId, ...)
        pub MyReceiveTasks get (fn my_receive_tasks): map hasher(blake2_128_concat) T::AccountId => BTreeSet<u128>;

	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
		/// 当任务被创建时触发此事件 [owner, task_id, task_status, stake_token, task_detail]
        TaskCreated(AccountId, u128, u8, u128, Vec<u8>),
        /// 当任务状态被更新触发此事件 [owner|bidder, task_id, task_status]
        TaskStatusUpdated(AccountId, u128, u8),
        /// 当任务被撤销时触发此事件 [owner, task_id]
		TaskRevoked(AccountId, u128),
        /// 当完成任务投标时触发此事件 [bidder, task_id, stake_token]
        BidCompleted(AccountId, u128, u128),
        /// 当完成任务选（中）标时触发此事件 [owner, bidder, task_id, stake_token]
        TaskDelegated(AccountId, AccountId, u128),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
		/// 该任务已经存在
		TaskAlreadyExisted,
        /// 该任务详情内容长度超出最大限制
        InvalidTaskDetail,
		/// 该任务不存在
		NoSuchTask,
		/// 该任务不是本人创建的
		NotTaskOwner,
        /// 不能投标自己的任务
        NotBidSelf,
        /// 该任务投标通道已关闭
        BidClosed,
        /// 同一人同一任务不能重复投标
        NoDuplicateBid,
        /// 该任务选（中）标通道已关闭
        DelegateClosed,
        /// 该用户没有投标这个任务
        NoSuchBidder,
        /// 该操作只有任务创建者或中标者才有权限
        NotTaskOwnerOrReceiver,
        /// 该操作只有任务中标者才有权限
        NotTaskReceiver,
        /// 无效的任务状态值
        InvalidTaskStatus,
        /// 质押token数量太少
        InvalidStakeToken,
        
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
        fn create_task(origin, task_id: u128, stake_token: u128, task_detail: Vec<u8>) -> dispatch::DispatchResult {
            // 检查当前操作者是否已签名
            // 如果未签名，则返回错误
            let sender = ensure_signed(origin)?;

            // 检查任务ID是否已存在
            ensure!(!TaskDetail::<T>::contains_key(&task_id), Error::<T>::TaskAlreadyExisted);

            // 检查质押token数量 必须大于0
            ensure!(stake_token > 0, Error::<T>::InvalidStakeToken);

            // 检查任务详情内容是否超过最大字节数
            //ensure!(task_detail.len() <= config::TASK_DETAIL_MAX_BYTES, Error::<T>::InvalidTaskDetail);

            // 从FRAME系统模块中获取当前区块高度
            let current_block = <frame_system::Module<T>>::block_number();

            // 初始任务状态
            let task_status = TaskStatus::Bidding as u8;

            // TODO 质押

            // 保存任务详情
            TaskDetail::<T>::insert(&task_id, (sender.clone(), task_detail.clone(), current_block));
            RelTaskStatus::insert(&task_id, task_status.clone());
            RelCreateTaskStake::insert(&task_id, stake_token.clone());

            // 更新我创建的任务列表
            let mut my_tasks = MyCreateTasks::<T>::get(&sender);
            my_tasks.insert(task_id.clone());
            MyCreateTasks::<T>::insert(&sender, my_tasks);

            // 任务总数+1
            let task_count = TaskCount::get();
            match task_count.checked_add(1) { 
                Some(v)=> { TaskCount::put(v); }, 
                None => (), 
            }   
            
            // 更新用户列表
            let mut user_list = UserList::<T>::get();
            if !user_list.contains(&sender) {
                user_list.insert(sender.clone());
                UserList::<T>::put(user_list);
            }

            // 更新任务列表
            let mut task_list = TaskList::get();
            if !task_list.contains(&task_id) {
                task_list.insert(task_id.clone());
                TaskList::put(task_list);
            }

            // 触发创建任务事件 
			Self::deposit_event(RawEvent::TaskCreated(sender, task_id, task_status, stake_token, task_detail));

			// Return a successful DispatchResult
			Ok(())
			
        }

        /// 更新任务状态        
        #[weight = 10_000]
        fn update_task_status(origin, task_id: u128, task_status: u8) -> dispatch::DispatchResult {
            // 检查调用者是否已签名
            // 如果未签名，则函数将返回错误
            let sender = ensure_signed(origin)?;

            // 检查任务是否存在
            ensure!(TaskDetail::<T>::contains_key(&task_id), Error::<T>::NoSuchTask);

            let current_task_status = RelTaskStatus::get(&task_id);

            // 检查任务状态值的有效性
            ensure!((task_status > current_task_status) //禁止任务状态回退
                 && (task_status == (TaskStatus::Bidding as u8)
                 || task_status  == (TaskStatus::Doing as u8)
                 || task_status  == (TaskStatus::UnDone as u8)
                 || task_status  == (TaskStatus::Delivered as u8)
                 || task_status  == (TaskStatus::Accepted as u8)
                 || task_status  == (TaskStatus::Arbitrating as u8) 
                 || task_status  == (TaskStatus::Judging as u8) 
                 || task_status  == (TaskStatus::Finished as u8)), Error::<T>::InvalidTaskStatus);

            // 获取任务创建者
            let (owner, _, _) = TaskDetail::<T>::get(&task_id);

            // 获取任务中标者
            let receiver = RelTaskReceiver::<T>::get(&task_id);

            // 检查操作权限
            // 任务状态的更新默认只有任务创建者才有权限
            // 但其中Completed只有任务中标者有权限操作，Arbitrating、Judging两个状态双方都有权限操作
            if task_status == (TaskStatus::Delivered as u8) {
                ensure!(sender == receiver, Error::<T>::NotTaskReceiver);
            } else if task_status == (TaskStatus::Arbitrating as u8) || task_status == (TaskStatus::Judging as u8) {
                ensure!((sender == owner || sender == receiver), Error::<T>::NotTaskOwnerOrReceiver);
            } else {
                ensure!(sender == owner, Error::<T>::NotTaskOwner);
            }            

            // 更新任务状态            
            RelTaskStatus::insert(&task_id, task_status.clone());

            // TODO 任务正常完成后 
            let finished = TaskStatus::Finished as u8;
            if finished == task_status {
                //TODO 自动解除质押、转账
            }

            // 触发修改任务事件
            Self::deposit_event(RawEvent::TaskStatusUpdated(sender, task_id, task_status));

			// Return a successful DispatchResult
			Ok(())

        }

        /// 投标任务
        #[weight = 10_000]
        fn bid_task(origin, task_id: u128, stake_token: u128) -> dispatch::DispatchResult {
            // 检查当前操作者是否已签名
            // 如果未签名，则返回错误
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;

            // 获取任务创建者
            let (owner, _, _) = TaskDetail::<T>::get(&task_id);

            // 检查当前操作者是否为任务的创建者 不能自己投标自己
            ensure!(sender != owner, Error::<T>::NotBidSelf);   
            
            // 检查任务是否存在
            ensure!(TaskDetail::<T>::contains_key(&task_id), Error::<T>::NoSuchTask);

            // 检查质押token是否满足要求：任务创建者质押数量的1/10
            let create_task_stake = RelCreateTaskStake::get(&task_id);
            ensure!(stake_token < (((create_task_stake as f32) * config::BID_STAKE_RATIO) as u128), Error::<T>::InvalidStakeToken);

            // 检查任务状态是否为投标中
            let task_status = RelTaskStatus::get(&task_id);
            ensure!((TaskStatus::Bidding as u8) == task_status, Error::<T>::BidClosed);

            // 检查是否已经投标过了
            ensure!(!RelBidTaskStake::<T>::contains_key(&task_id, &sender), Error::<T>::NoDuplicateBid);

            // TODO 质押token

            // 保存投标任务与质押的Token数量关系
            RelBidTaskStake::<T>::insert(&task_id, &sender, stake_token.clone());

            // 更新任务与投标人关系
            let mut task_bidder = RelTaskBidder::<T>::get(&task_id);
            if !task_bidder.contains(&sender) {
                task_bidder.insert(sender.clone());
                RelTaskBidder::<T>::insert(&task_id, task_bidder);
            }

            // 更新我投标的任务列表
            let mut my_bid_tasks = MyBidTasks::<T>::get(&sender);
            if !my_bid_tasks.contains(&task_id) {
                my_bid_tasks.insert(task_id.clone());
                MyBidTasks::<T>::insert(&sender, my_bid_tasks);
            } 

            // 触发投标任务事件
            Self::deposit_event(RawEvent::BidCompleted(sender, task_id, stake_token));

            // Return a successful DispatchResult
			Ok(())

        }

        /// 任务选标（中标）
        #[weight = 10_000]
        fn delegate_task(origin, bidder: T::AccountId, task_id: u128) -> dispatch::DispatchResult {
            // 检查当前操作者是否已签名
            // 如果未签名，则返回错误
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;

            // 检查任务是否存在
            ensure!(TaskDetail::<T>::contains_key(&task_id), Error::<T>::NoSuchTask);

            // 获取任务创建者
            let (owner, _, _) = TaskDetail::<T>::get(&task_id);

            // 检查当前操作者是否为任务创建者
            ensure!(sender == owner, Error::<T>::NotTaskOwner);

            // 检查任务状态是否为投标中 RelTaskStatus
            let task_status = RelTaskStatus::get(&task_id);
            ensure!((TaskStatus::Bidding as u8) == task_status, Error::<T>::DelegateClosed);

            // 检查bidder是否已投标 忽略：检查bidder在本任务上是否已质押
            let task_bidder = RelTaskBidder::<T>::get(&task_id);
            ensure!(task_bidder.contains(&bidder), Error::<T>::NoSuchBidder);

            // 保存任务的中标人
            RelTaskReceiver::<T>::insert(&task_id, bidder.clone());

            // 更新任务状态 进入Doing状态
            let task_status = TaskStatus::Doing as u8;
            RelTaskStatus::insert(&task_id, task_status);

            // 更新我中标的任务列表
            let mut my_receive_tasks = MyReceiveTasks::<T>::get(&bidder);
            if !my_receive_tasks.contains(&task_id) {
                my_receive_tasks.insert(task_id.clone());
                MyReceiveTasks::<T>::insert(&bidder, my_receive_tasks);
            }
            
            // 退还未中标人的质押、清空RelBidTaskStake为0（除中标人外）

            // 触发选标任务事件
            Self::deposit_event(RawEvent::TaskDelegated(sender, bidder, task_id));

            // Return a successful DispatchResult
			Ok(())

        }

        /// 撤销任务
        #[weight = 10_000]
        fn revoke_task(origin, task_id: u128) -> dispatch::DispatchResult {
            // 检查当前操作者是否已签名
            // 如果未签名，则返回错误
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;

            // 检查任务是否存在
            ensure!(TaskDetail::<T>::contains_key(&task_id), Error::<T>::NoSuchTask);

            // 获取任务创建者
            let (owner, _, _) = TaskDetail::<T>::get(&task_id);

            // 检查当前操作者是否为任务创建者
            ensure!(sender == owner, Error::<T>::NotTaskOwner);

            // 从链上存储中撤销任务
            TaskDetail::<T>::remove(&task_id);

            // 更新我的任务集
            let mut my_create_tasks = MyCreateTasks::<T>::get(&sender);
            if my_create_tasks.contains(&task_id) {
                my_create_tasks.remove(&task_id); //移除指定task_id
            }

            // TODO 退还本人与所有投标人的质押
            
            // 任务总数-1
            TaskCount::mutate(|v| *v -= 1);

            // 触发撤销任务事件
            Self::deposit_event(RawEvent::TaskRevoked(sender, task_id));

            // Return a successful DispatchResult
            Ok(())

        }

		
	}
}
