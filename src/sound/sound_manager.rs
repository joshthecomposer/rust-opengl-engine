#![allow(dead_code)]

use std::{cell::Cell, collections::HashMap, ffi::CString};

use crate::{animation::animation::Animation, config::game_config::GameConfig, sound::fmod::FMOD_Studio_EventDescription_LoadSampleData};

use super::fmod::{FMOD_Studio_EventDescription_CreateInstance, FMOD_Studio_EventInstance_SetParameterByName, FMOD_Studio_EventInstance_Start, FMOD_Studio_EventInstance_Stop, FMOD_Studio_System_Create, FMOD_Studio_System_GetEvent, FMOD_Studio_System_Initialize, FMOD_Studio_System_LoadBankFile, FMOD_Studio_System_Update, FMOD_INIT_NORMAL, FMOD_STUDIO_BANK, FMOD_STUDIO_EVENTDESCRIPTION, FMOD_STUDIO_EVENTINSTANCE, FMOD_STUDIO_INIT_NORMAL, FMOD_STUDIO_SYSTEM, FMOD_VERSION};

pub struct SoundData {
    description: FMOD_STUDIO_EVENTDESCRIPTION,
    instance: FMOD_STUDIO_EVENTINSTANCE,
}

#[derive(Clone, Debug)]
pub struct SoundTrigger {
    pub sound_type: String, //TODO: JW - This sound_type should probably be an enum of SoundType?? Maybe?
    pub frame: usize,
}

#[derive(Clone, Debug)]
pub struct OneShot {
    pub sound_type: String, 
    pub segment: u32,
    pub triggered: Cell<bool>,
}

pub struct SoundManager {
    pub fmod_system: FMOD_STUDIO_SYSTEM,
    pub sounds: HashMap<String, SoundData>, //The key (String) is the sound_name in the game_config.json
    pub playing_bg: bool,
    pub master_volume: f32,
} 

impl SoundManager {
    pub fn new(config: &GameConfig) -> SoundManager {
        let sound_props = &config.sounds;

        let mut fmod_system: FMOD_STUDIO_SYSTEM = std::ptr::null_mut();
        let mut sounds = HashMap::new();

        unsafe {
            /************** INITIALIZE FMOD SYSTEM ******************/
            println!("FMOD VERSION IS {}", FMOD_VERSION);
            let result = FMOD_Studio_System_Create(&mut fmod_system, FMOD_VERSION);
            if result != 0 {
                panic!("FMOD System creation failed with error code {}", result);
            }

            let result = FMOD_Studio_System_Initialize(
                fmod_system, 
                512, 
                FMOD_STUDIO_INIT_NORMAL, 
                FMOD_INIT_NORMAL, 
                std::ptr::null_mut(), 
            );
            if result != 0 {
                panic!("FMOD System initialization failed with error code {}", result);
            }

            /************** LOAD BANK AND BANK STRINGS ******************/
            let bank_path = CString::new("resources/fmod/Desktop/Master.bank").expect("CString::new failed");
            let mut bank: FMOD_STUDIO_BANK = std::ptr::null_mut();
            let result = FMOD_Studio_System_LoadBankFile(
                fmod_system,
                bank_path.as_ptr(),
                0,
                &mut bank,
            );

            if result != 0 {
                panic!("FMOD Bank load failed with error code {}", result);
            }

            let strings_bank_path = CString::new("resources/fmod/Desktop/Master.strings.bank").expect("CString::new failed");
            let mut strings_bank: FMOD_STUDIO_BANK = std::ptr::null_mut();
            let result = FMOD_Studio_System_LoadBankFile(
                fmod_system,
                strings_bank_path.as_ptr(),
                0,
                &mut strings_bank,
            );

            if result != 0 {
                panic!("FMOD Strings Bank load failed with error code {}", result);
            }

            /***************** CREATE EVENT DESC AND INSTANCES ****************/


            for (sound_name, path) in sound_props {
                let event_path = CString::new(path.as_str()).expect("CString::new failed");
                let mut description: FMOD_STUDIO_EVENTDESCRIPTION = std::ptr::null_mut();
                let mut instance: FMOD_STUDIO_EVENTINSTANCE = std::ptr::null_mut();

                let result = FMOD_Studio_System_GetEvent(
                    fmod_system, 
                    event_path.as_ptr(), 
                    &mut description
                );
                if result != 0 {
                    panic!("Failed to load the event for {:?} with code {}", sound_name, result);
                }

                let result = FMOD_Studio_EventDescription_CreateInstance(
                    description, 
                    &mut instance
                );
                if result != 0 {
                    // Handle error
                    panic!("Failed to create event instance {}", result);
                }
                FMOD_Studio_EventDescription_LoadSampleData(description);
                sounds.insert(sound_name.to_string(), SoundData {description, instance});
            }
        }
        SoundManager {
            fmod_system,
            sounds,
            playing_bg: false,
            master_volume: 1.0,
        }

    }

    pub fn update(&self) {
        unsafe {
            let result = FMOD_Studio_System_Update(self.fmod_system);
            if result != 0 {
                eprintln!("FMOD update failed with error code {}", result);
            }
        }

    }

    pub fn play_sound(&mut self, sound_type: String){
        let sound_data = self.sounds.get(&sound_type).unwrap();
        unsafe {
            let result = FMOD_Studio_EventInstance_Start(sound_data.instance);
            if result != 0 {
                    eprintln!("FMOD sound failed with error code {}", result);
            }
        }
    }
    pub fn stop_sound(&mut self, sound_type: String){
        let sound_data = self.sounds.get(&sound_type).unwrap();
        unsafe {
            FMOD_Studio_EventInstance_Stop(sound_data.instance, super::fmod::FMOD_STUDIO_STOP_MODE::FMOD_STUDIO_STOP_IMMEDIATE);
        }
    }

    pub fn set_master_volume(&mut self) {
        println!("Setting master volume to {}", self.master_volume);
        let sound_data = self.sounds.get("music").unwrap();
        let vol = CString::new("main_volume").unwrap();
        unsafe {
            let result = FMOD_Studio_EventInstance_SetParameterByName(sound_data.instance, vol.as_ptr(), self.master_volume, 0);
            if result != 0 {
                println!("Updating volume failed with error code: {}", result);
            }
        }
    }
}
