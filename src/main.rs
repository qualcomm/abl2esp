// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

#![no_main]
#![no_std]

use core::mem::MaybeUninit;
use core::ptr::NonNull;

use log::info;
use uefi::boot::{self, EventType, LoadImageSource, OpenProtocolParams, SearchType, Tpl, image_handle};
use uefi::proto::console::text::{Input, Key, ScanCode};
use uefi::proto::device_path::{build, DevicePath};
use uefi::proto::media::file::{File, FileAttribute, FileMode};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::proto::BootPolicy;
use uefi::runtime::ResetType;
use uefi::{guid, prelude::*, CStr16};
use uefi::{Identify, Result};

const BOOTAA64_PATH: &CStr16 = cstr16!(r"\EFI\BOOT\BOOTAA64.EFI");

fn connect_all() -> Result {
    let handles = boot::locate_handle_buffer(SearchType::AllHandles)?;
    for handle in handles.iter() {
        let _ = boot::connect_controller(*handle, None, None, true);
    }

    Ok(())
}

fn load_bootaa64() -> Result<Option<Handle>> {
    let handles = boot::locate_handle_buffer(SearchType::ByProtocol(&SimpleFileSystem::GUID))?;
    for handle in handles.iter() {
        let device_path_protocol = boot::open_protocol_exclusive::<DevicePath>(*handle)?;

        let filesystem = boot::open_protocol_exclusive::<SimpleFileSystem>(*handle);
        let Ok(mut filesystem) = filesystem else {
            info!("Unable to open SimpleFileSystem protocol on handle");
            continue;
        };

        let volume = filesystem.open_volume();
        let Ok(mut volume) = volume else {
            info!("Unable to open volume");
            continue;
        };

        let bootaa64 = volume.open(BOOTAA64_PATH, FileMode::Read, FileAttribute::READ_ONLY);
        if bootaa64.is_ok() {
            let mut path_iterator = device_path_protocol.node_iter();

            let mut path_buf = [MaybeUninit::uninit(); 256];
            let path: &DevicePath = build::DevicePathBuilder::with_buf(&mut path_buf)
                .push(&path_iterator.next().unwrap())
                .unwrap()
                .push(&path_iterator.next().unwrap())
                .unwrap()
                .push(&build::media::FilePath {
                    path_name: BOOTAA64_PATH,
                })
                .unwrap()
                .finalize()
                .unwrap();

            let image = boot::load_image(
                boot::image_handle(),
                LoadImageSource::FromDevicePath {
                    device_path: path,
                    boot_policy: BootPolicy::ExactMatch,
                },
            )
            .expect("Failed to load image");

            return Ok(Some(image));
        }
    }

    Ok(None)
}

const READY_TO_BOOT: uefi::Guid = guid!("7ce88fb3-4bd7-4679-87a8-a8d8dee50d2b");
const END_OF_DXE: uefi::Guid = guid!("02ce967a-dd7e-4ffc-9ee7-810cf0470880");
const EFI_EVENT_DETECT_SD_CARD: uefi::Guid = guid!("b7972c36-8a4c-4a56-8b02-1159b52d4bfb");

fn signal_guid(guid: &uefi::Guid) -> Result {
    unsafe extern "efiapi" fn callback(_: uefi::Event, _: Option<NonNull<core::ffi::c_void>>) {}

    let guid: Option<NonNull<uefi::Guid>> = NonNull::new(&mut guid.clone());
    let event = unsafe {
        boot::create_event_ex(
            EventType::NOTIFY_SIGNAL,
            Tpl::NOTIFY,
            Some(callback),
            None,
            guid,
        )
    }
    .expect("Failed to create event");

    boot::signal_event(&event)?;
    boot::close_event(event)?;

    Ok(())
}

fn get_held_key() -> Result<Option<Key>> {
    let handles = boot::locate_handle_buffer(SearchType::ByProtocol(&Input::GUID))?;
    for handle in handles.iter() {
        let mut input = unsafe { 
            boot::open_protocol::<Input>( OpenProtocolParams {
                handle: *handle,
                agent: image_handle(),
                controller: None,
            }, 
            boot::OpenProtocolAttributes::GetProtocol) 
        }?;

        if let Ok(Some(pressed_key)) = input.read_key() {
            return Ok(Some(pressed_key));
        }
    }

    Ok(None)
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    signal_guid(&EFI_EVENT_DETECT_SD_CARD).expect("Failed to signal SD-card detect");

    connect_all().expect("Failed to connect drivers");

    signal_guid(&READY_TO_BOOT).expect("Failed to signal ReadyToBoot");

    signal_guid(&END_OF_DXE).expect("Failed to signal end of DXE");

    if let Ok(Some(Key::Special(ScanCode::ESCAPE))) = get_held_key() {
        runtime::reset(
            ResetType::PLATFORM_SPECIFIC,
            Status::SUCCESS,
            Some(cstr16!("EDL").as_bytes()));
    }

    let image = load_bootaa64()
        .expect("An error occurred while searching for bootaa64.efi")
        .expect("No bootaa64.efi found");

    info!("Found bootaa64.efi, starting..");
    boot::start_image(image).expect("Failed to start bootaa64.efi");

    boot::stall(10_000_000);

    Status::NOT_FOUND
}
