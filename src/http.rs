//! Models to deserialize responses into and functions to create `http` crate
//! requests.

use http::{
    header::{HeaderValue, AUTHORIZATION},
    Error as HttpError, Request,
};
use std::{
    convert::TryFrom,
    net::IpAddr,
};
use percent_encoding::NON_ALPHANUMERIC;
use serde::{Deserialize, Deserializer, Serialize};

/// The type of search result given.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LoadType {
    /// Loading the results failed.
    LoadFailed,
    /// There were no matches.
    NoMatches,
    /// A playlist was found.
    PlaylistLoaded,
    /// Some results were found.
    SearchResult,
    /// A single track was found.
    TrackLoaded,
}

/// A track within a search result.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct Track {
    /// Details about a track, such as the author and title.
    pub info: TrackInfo,
    /// The base64 track string that you use in the [`Play`] event.
    ///
    /// [`Play`]: crate::model::outgoing::Play
    pub track: String,
}

/// Additional information about a track, such as the author.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct TrackInfo {
    /// The name of the author, if provided.
    pub author: Option<String>,
    /// The identifier of the source of the track.
    pub identifier: String,
    /// Whether the source is seekable.
    pub is_seekable: bool,
    /// Whether the source is a stream.
    pub is_stream: bool,
    /// The length of the audio in milliseconds.
    pub length: u64,
    /// The position of the audio.
    pub position: u64,
    /// The title, if provided.
    pub title: Option<String>,
    /// The source URI of the track.
    pub uri: String,
}

/// Information about a playlist from a search result.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct PlaylistInfo {
    /// The name of the playlist, if available.
    pub name: Option<String>,
    /// The selected track within the playlist, if available.
    #[serde(default, deserialize_with = "deserialize_selected_track")]
    pub selected_track: Option<u64>,
}

// Any negative value should be treated as None.
fn deserialize_selected_track<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<i64>::deserialize(deserializer)
        .ok()
        .flatten()
        .and_then(|selected| u64::try_from(selected).ok()))
}

/// Possible track results for a query.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct LoadedTracks {
    /// The type of search result, such as a list of tracks or a playlist.
    pub load_type: LoadType,
    /// Information about the playlist, if provided.
    pub playlist_info: PlaylistInfo,
    /// The list of tracks returned for the search query.
    pub tracks: Vec<Track>,
}

/// A failing IP address within the planner.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct FailingAddress {
    /// The IP address.
    pub address: String,
    /// The time that the address started failing in unix time.
    pub failing_timestamp: u64,
    /// The time that the address started failing as a timestamp.
    pub failing_time: String,
}

/// The IP version in use by the block.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
pub enum IpBlockType {
    /// An IPv4 block type.
    #[serde(rename = "Inet4Address")]
    Inet4,
    /// An IPv6 block type.
    #[serde(rename = "Inet6Address")]
    Inet6,
}

/// A block of IP addresses.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
pub struct IpBlock {
    /// The IP version of the IP block.
    pub kind: IpBlockType,
    /// The size of the block's addresses.
    pub size: u64,
}

/// The type of route planner in use.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "PascalCase")]
pub enum RoutePlannerType {
    /// A Nano IP route planner.
    NanoIp,
    /// A Rotating IP route planner.
    RotatingIp,
    /// A Rotating Nano IP route planner.
    RotatingNanoIp,
}

/// The route planner in use.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum RoutePlanner {
    /// Information about a Nano IP route planner.
    NanoIp(NanoIpRoutePlanner),
    /// Information about a Rotating IP route planner.
    RotatingIp(RotatingIpRoutePlanner),
    /// Information about a Rotating Nano IP route planner.
    RotatingNanoIp(RotatingNanoIpRoutePlanner),
}

/// A Nano IP planner.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct NanoIpRoutePlanner {
    /// The type of planner that is currently active.
    ///
    /// For this planner, this is always [`RoutePlannerType::NanoIp`]
    pub class: RoutePlannerType,
    /// The details of the currently active Nano IP route planner.
    pub details: NanoIpDetails,
}

/// Information about a Nano IP planner.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct NanoIpDetails {
    /// The active offset within the IP block.
    pub current_address_index: u64,
    /// A list of IP addresses in the range that are failing.
    pub failing_addresses: Vec<FailingAddress>,
    /// The associated IP block.
    pub ip_block: IpBlock,
}

/// A Rotating IP planner.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct RotatingIpRoutePlanner {
    /// The type of planner that is currently active.
    ///
    /// For this planner, this is always [`RoutePlannerType::RotatingIp`]
    pub class: RoutePlannerType,
    /// The details of the currently active rotating IP route planner.
    pub details: RotatingIpDetails,
}

/// Information about a Rotating IP planner.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct RotatingIpDetails {
    /// The currently used IP address.
    pub current_address: String,
    /// A list of IP addresses in the range that are failing.
    pub failing_addresses: Vec<FailingAddress>,
    /// The associated IP block.
    pub ip_block: IpBlock,
    /// The current offset used within the IP block.
    pub ip_index: u64,
    /// The number of rotations that have happened since the server started.
    pub rotate_index: u64,
}

/// A Rotating Nano IP planner.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct RotatingNanoIpRoutePlanner {
    /// The type of planner that is currently active.
    ///
    /// For this planner, this is always [`RoutePlannerType::RotatingNanoIp`]
    pub class: RoutePlannerType,
    /// The details of the currently active rotating nano IP route planner.
    pub details: RotatingNanoIpDetails,
}

/// Information about a Rotating Nano IP planner.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct RotatingNanoIpDetails {
    /// The block IPs that are chosen.
    pub block_index: String,
    /// The current IP address on rotation.
    pub current_address_index: u64,
    /// A list of IP addresses in the range that are failing.
    pub failing_addresses: Vec<FailingAddress>,
    /// The associated IP block.
    pub ip_block: IpBlock,
}

/// Get a list of tracks that match an identifier.
///
/// The response will include a body which can be deserialized into a
/// [`LoadedTracks`].
///
/// # Errors
///
/// See the documentation for [`http::Error`].
pub fn load_track(
    address: String,
    identifier: impl AsRef<str>,
    authorization: impl AsRef<str>,
) -> Result<Request<&'static [u8]>, HttpError> {
    let identifier =
        percent_encoding::percent_encode(identifier.as_ref().as_bytes(), NON_ALPHANUMERIC);
    let url = format!("http://{address}/loadtracks?identifier={identifier}");

    let mut req = Request::get(url);

    let auth_value = HeaderValue::from_str(authorization.as_ref())?;
    req = req.header(AUTHORIZATION, auth_value);

    req.body(b"")
}

/// Get the configured route planner for a node by address.
///
/// The response will include a body which can be deserialized into a
/// [`RoutePlanner`].
///
/// # Errors
///
/// See the documentation for [`http::Error`].
pub fn get_route_planner(
    address: String,
    authorization: impl AsRef<str>,
) -> Result<Request<&'static [u8]>, HttpError> {
    let mut req = Request::get(format!("{address}/routeplanner/status"));

    let auth_value = HeaderValue::from_str(authorization.as_ref())?;
    req = req.header(AUTHORIZATION, auth_value);

    req.body(b"")
}

/// Unmark an IP address as being failed, meaning that it can be used again.
///
/// The response will not include a body on success.
///
/// # Errors
///
/// See the documentation for [`http::Error`].
pub fn unmark_failed_address(
    node_address: String,
    authorization: impl AsRef<str>,
    route_address: impl Into<IpAddr>,
) -> Result<Request<Vec<u8>>, HttpError> {
    let mut req = Request::post(format!("{}/routeplanner/status", node_address));

    let auth_value = HeaderValue::from_str(authorization.as_ref())?;
    req = req.header(AUTHORIZATION, auth_value);

    req.body(
        serde_json::to_vec(&serde_json::json!({
            "address": route_address.into(),
        }))
        .expect("valid json"),
    )
}
