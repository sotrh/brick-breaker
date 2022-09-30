use rand::prelude::*;
use std::{collections::HashMap, fs::File, io::{BufReader, Read, Cursor}};

pub struct SoundSystem {
    rng: rand::rngs::ThreadRng,
    current_sink: usize,
    sinks: Vec<rodio::Sink>,
    banks: HashMap<String, SoundBank>,
    #[allow(dead_code)]
    device: rodio::OutputStream,
    #[allow(dead_code)]
    handle: rodio::OutputStreamHandle,
}

impl SoundSystem {
    pub fn with_json(json: &str) -> anyhow::Result<Self> {
        let atlas: HashMap<String, SoundDef> = serde_json::from_str(json)?;
        let rng = rand::thread_rng();

        let (device, handle) = rodio::OutputStream::try_default().unwrap();
        let sinks = (0..8)
            .map(|_| { 
                let sink = rodio::Sink::try_new(&handle)?;
                Ok(sink)
            })
            .collect::<anyhow::Result<Vec<rodio::Sink>>>()?;

        let mut banks = HashMap::new();
        for (k, v) in atlas.into_iter() {
            let sources = v
                .files
                .into_iter()
                .map(|f| {
                    let mut reader = BufReader::new(File::open(f)?);
                    let mut buffer = Vec::new();
                    reader.read_to_end(&mut buffer)?;
                    Ok(Cursor::new(buffer))
                })
                .collect::<anyhow::Result<Vec<_>>>()?;
            banks.insert(k, SoundBank { sources });
        }

        Ok(Self {
            rng,
            current_sink: 0,
            sinks,
            banks,
            device,
            handle,
        })
    }

    pub fn play_sound(&mut self, name: &str) {
        if let Some(bank) = self.banks.get(name) {
            let src = bank.random(&mut self.rng).clone();
            let decoder = rodio::Decoder::new(src).unwrap();
            let sink = &self.sinks[self.current_sink];
            sink.append(decoder);
            self.current_sink = (self.current_sink + 1) % self.sinks.len();
        }
    }

}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SoundDef {
    files: Vec<String>,
}

pub struct SoundBank {
    sources: Vec<Cursor<Vec<u8>>>,
}

impl SoundBank {
    pub fn random(&self, rng: &mut rand::rngs::ThreadRng) -> &Cursor<Vec<u8>> {
        let i = rng.gen_range(0..self.sources.len());
        &self.sources[i]
    }
}
