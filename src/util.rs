use csv;
use std::error::Error;
use serde::Deserialize;
use std::vec::Vec;

use solana_sdk::{
    // instruction::{AccountMeta, },
    // pubkey::Pubkey,
    signature::{read_keypair_file, Keypair},
};
use std::fs;
use serde_json;
use serde_json::{json, Value};
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient, TryFromUri};
use std::io::Cursor;
use http::uri::{Scheme};

const ORIGIN_DIR: &str = "assets_original";
const DIR: &str = "assets";

#[derive(Debug, Deserialize, Clone)]
pub struct Record {
    pub name: String,
    pub id: String,
    pub cert_type: String,
    pub address: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Record2 {
    pub id: String,
    pub address: String,
    pub ipfs_hash: String,
}


// pub fn get_pub(pubkey: &str) -> Pubkey {
//     Pubkey::from_str(pubkey).unwrap()
// }
// pub fn getkey(public_key: Pubkey, is_signer: bool, is_writable: bool) -> AccountMeta {
//     if is_writable {
//         AccountMeta::new(public_key, is_signer)
//     } else {
//         AccountMeta::new_readonly(public_key, is_signer)
//     }
// }

pub fn load_config_keypair() -> Keypair {
    let config_path = solana_cli_config::CONFIG_FILE.as_ref().unwrap();
    let cli_config =
        solana_cli_config::Config::load(config_path).expect("failed to load config file");
    read_keypair_file(cli_config.keypair_path).expect("failed to load keypair")
}

pub async fn read_from_file(path: &str) -> Result<Vec<Record2>, Box<dyn Error>> {
    // Creates a new csv `Reader` from a file
    let mut reader = csv::Reader::from_path(path)?;

    // Retrieve and print header record
    let headers = reader.headers()?;
    println!("{:?}", headers);

    // `.deserialize` returns an iterator of the internal
    // record structure deserialized

    let mut vec: Vec<Record2> = Vec::new();
    let mut i = 0;
    let client = IpfsClient::from_host_and_port(Scheme::HTTPS, "ipfs.infura.io", 5001).unwrap();
    // let client = IpfsClient::default();

    for result in reader.deserialize() {
        let record: Record = result?;
        // println!("{:?}", record);
        // println!("{:?}", record.name);
        // println!("{:?}", record.id);
        // println!("{:?}", record.cert_type);
        // println!("{:?}", record.address);
        
        let rec: Record = Record {
            name: record.name,
            id: record.id,
            cert_type: record.cert_type,
            address: record.address.clone()
        };

        let json_data = make_json_metadata(rec, i).await;

        println!("{}.json ipfs uploading", i);
        match client.add(Cursor::new(json_data.to_string())).await {
            Ok(res) => {
                println!("{}", res.hash);
                let rec: Record2 = Record2 {
                    id: i.to_string(),
                    address: record.address,
                    ipfs_hash: res.hash
                };
                vec.push(rec);
                // let _res = client.pin_add(&res.hash[..], false).await.unwrap();
            },
            Err(e) => eprintln!("error adding file: {}", e)
        }
        // client.pin_add("QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ", true);
    
        i+=1;
    }

    Ok(vec)
}

async fn make_json_metadata(record: Record, i: i32) -> Value {
    let file_name = format!("{}-{}-{}", record.cert_type, record.name, record.id);
    // reset_folder(DIR.to_string());
    println!("{}/{}.jpg", ORIGIN_DIR, file_name);
    fs::rename(format!("{}/{}.jpg", ORIGIN_DIR, file_name), format!("{}/{}.jpg", DIR, file_name)).unwrap();
    
    let client = IpfsClient::from_str("https://ipfs.infura.io:5001").unwrap();
    // let client = IpfsClient::default();
    // let data = Cursor::new(fs::read(format!("{}/{}.jpg", DIR, file_name)));
    match client.add_path(format!("{}/{}.jpg", DIR, file_name)).await {
        Ok(res) => println!("{:?}", res),
        Err(e) => eprintln!("error adding file: {}", e)
    }
    // match client.add(Cursor::new(json_data.to_string())).await {
    //         Ok(res) => {
    //             println!("{}", res.hash);
    //         }
    //     }

    let json_data = json!({
        "name": "The Z Institute Certificate",
        "symbol": "Z_CERT",
        "description": "This serves as the certificate of The Z Institute's DeFi Accelerator Batch #2.",
        "seller_fee_basis_points": 1000,
        "image": format!("{}.jpg", i),
        "external_url": "https://zinstitute.net",
        "attributes": [
            {
                "trait_type": "Student ID",
                "value": record.id
            },
            {
                "trait_type": "Course",
                "value": "DeFi Accelerator",
            },
            {
                "trait_type": "Batch",
                "value": "2",
            },
            {
                "trait_type": "Name",
                "value": record.name,
            },
            {
                "trait_type": "Award",
                "value": record.cert_type,
            },
            {
                "trait_type": "Term",
                "value": "2022/01",
            },
        ], 
        "collection": {
            "name": "DeFi Accelerator Batch #2",
            "family": "The Z Institute",
        },
        "properties": {
        "files": [
          {
            "uri": format!("{}.jpg", i),
            "type": "image/jpg",
          },
        ],
        "creators": [
          {
            "address": "HXcdCwwu1wkS882Gs8rRV6f83MyestRyB5HmWGwuiFiq",
            "verified": false,
            "share": 100,
          },
        ],
      }
    });

    fs::write(format!("assets/{}.json", i), &json_data.to_string()).unwrap();
    println!("{}.json saved", i);

    json_data
}

fn reset_folder(dir_name: String) {
    fs::remove_dir_all(&dir_name).unwrap();
    fs::create_dir(dir_name).unwrap();
}