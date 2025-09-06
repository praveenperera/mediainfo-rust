use crate::ffi::{MediaInfo, MediaInfoError, MediaInfoInfo, MediaInfoResult, MediaInfoStream};
use chrono::{DateTime, NaiveDateTime, Utc};

use std::sync::{Arc, Mutex};
use std::time::Duration;

macro_rules! stream_struct {
    ($struct_name: ident) => {
        #[derive(Clone, Debug)]
        pub struct $struct_name {
            pub stream_type: MediaInfoStream,
            pub index: usize,
            pub handler: Arc<Mutex<MediaInfo>>,
        }
    };
}

macro_rules! base_stream_implement {
    ($struct_name: ident) => {
        impl BaseStream for $struct_name {
            fn stream_type(&self) -> MediaInfoStream {
                self.stream_type
            }

            fn index(&self) -> usize {
                self.index
            }

            fn handler(&self) -> Option<&Arc<Mutex<MediaInfo>>> {
                Some(&self.handler)
            }
        }
    };
}

macro_rules! mediainfo_attr {
    ($meth_name: ident, $attr_name: tt) => {
        pub fn $meth_name(&self) -> MediaInfoResult<String> {
            match self.handler() {
                Some(arc) => arc.lock().unwrap().get(
                    self.stream_type(),
                    self.index(),
                    $attr_name,
                    MediaInfoInfo::Text,
                    MediaInfoInfo::Name,
                ),
                None => Err(MediaInfoError::NoDataOpenError),
            }
        }
    };
}

macro_rules! mediainfo_date {
    ($meth_name: ident, $attr_name: tt) => {
        pub fn $meth_name(&self) -> MediaInfoResult<DateTime<Utc>> {
            match self.handler() {
                Some(arc) => self.result_to_date(arc.lock().unwrap().get(
                    self.stream_type(),
                    self.index(),
                    $attr_name,
                    MediaInfoInfo::Text,
                    MediaInfoInfo::Name,
                )),
                None => Err(MediaInfoError::NoDataOpenError),
            }
        }
    };
}

macro_rules! mediainfo_i64 {
    ($meth_name: ident, $attr_name: tt) => {
        pub fn $meth_name(&self) -> MediaInfoResult<i64> {
            match self.handler() {
                Some(arc) => self.result_to_i64(arc.lock().unwrap().get(
                    self.stream_type(),
                    self.index(),
                    $attr_name,
                    MediaInfoInfo::Text,
                    MediaInfoInfo::Name,
                )),
                None => Err(MediaInfoError::NoDataOpenError),
            }
        }
    };
}

macro_rules! mediainfo_duration {
    ($meth_name: ident, $attr_name: tt) => {
        pub fn $meth_name(&self) -> MediaInfoResult<Duration> {
            match self.handler() {
                Some(arc) => self.result_to_duration(arc.lock().unwrap().get(
                    self.stream_type(),
                    self.index(),
                    $attr_name,
                    MediaInfoInfo::Text,
                    MediaInfoInfo::Name,
                )),
                None => Err(MediaInfoError::NoDataOpenError),
            }
        }
    };
}

#[derive(Clone, Debug)]
pub struct GeneralStream {
    pub stream_type: MediaInfoStream,
    pub handler: Option<Arc<Mutex<MediaInfo>>>,
}

pub trait BaseStream {
    fn stream_type(&self) -> MediaInfoStream;
    fn index(&self) -> usize;
    fn handler(&self) -> Option<&Arc<Mutex<MediaInfo>>>;

    fn result_to_duration(&self, result: MediaInfoResult<String>) -> MediaInfoResult<Duration> {
        match result?.parse::<u64>() {
            Ok(x) => Ok(Duration::from_millis(x)),
            Err(_) => Err(MediaInfoError::NonNumericResultError),
        }
    }

    fn result_to_i64(&self, result: MediaInfoResult<String>) -> MediaInfoResult<i64> {
        match result?.parse::<i64>() {
            Ok(x) => Ok(x),
            Err(_) => Err(MediaInfoError::NonNumericResultError),
        }
    }

    fn result_to_date(&self, result: MediaInfoResult<String>) -> MediaInfoResult<DateTime<Utc>> {
        let input = &result?;
        println!("input {input:?}");

        let naive = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S UTC");
        println!("naive {naive:?}");
        match naive {
            Ok(x) => Ok(DateTime::<Utc>::from_naive_utc_and_offset(x, Utc)),
            Err(_) => Err(MediaInfoError::NonNumericResultError),
        }
    }
}

impl BaseStream for GeneralStream {
    fn stream_type(&self) -> MediaInfoStream {
        self.stream_type
    }

    fn index(&self) -> usize {
        0
    }

    fn handler(&self) -> Option<&Arc<Mutex<MediaInfo>>> {
        self.handler.as_ref()
    }
}

stream_struct!(VideoStream);
base_stream_implement!(VideoStream);
stream_struct!(AudioStream);
base_stream_implement!(AudioStream);
stream_struct!(TextStream);
base_stream_implement!(TextStream);
stream_struct!(OtherStream);
base_stream_implement!(OtherStream);
stream_struct!(ImageStream);
base_stream_implement!(ImageStream);
stream_struct!(MenuStream);
base_stream_implement!(MenuStream);

/* GeneralStream */
impl GeneralStream {
    // Stream identification
    mediainfo_attr!(codec_id, "CodecID");
    mediainfo_attr!(format, "Format");
    mediainfo_attr!(format_profile, "Format_Profile");
    mediainfo_attr!(format_info, "Format_Info");
    mediainfo_attr!(codec, "Codec");

    // Encoding application info
    mediainfo_attr!(encoded_application_string, "Encoded_Application/String");
    mediainfo_attr!(encoded_application, "Encoded_Application");
    mediainfo_attr!(encoded_library, "Encoded_Library");

    // Metadata
    mediainfo_attr!(artist, "Artist");
    mediainfo_attr!(performer, "Performer");
    mediainfo_attr!(title, "Title");
    mediainfo_attr!(copyright, "Copyright");
    mediainfo_attr!(genre, "Genre");
    mediainfo_attr!(album, "Album");
    mediainfo_attr!(year, "Year");

    // Stream counts
    mediainfo_i64!(general_count, "GeneralCount");
    mediainfo_i64!(video_count, "VideoCount");
    mediainfo_i64!(audio_count, "AudioCount");
    mediainfo_i64!(text_count, "TextCount");
    mediainfo_i64!(other_count, "OtherCount");
    mediainfo_i64!(image_count, "ImageCount");
    mediainfo_i64!(menu_count, "MenuCount");
    mediainfo_i64!(audio_channels_total, "Audio_Channels_Total");

    // Format lists
    mediainfo_attr!(video_format_list, "Video_Format_List");
    mediainfo_attr!(video_format_with_hint_list, "Video_Format_WithHint_List");
    mediainfo_attr!(video_language_list, "Video_Language_List");
    mediainfo_attr!(audio_format_list, "Audio_Format_List");
    mediainfo_attr!(audio_format_with_hint_list, "Audio_Format_WithHint_List");
    mediainfo_attr!(audio_language_list, "Audio_Language_List");
    mediainfo_attr!(text_format_list, "Text_Format_List");
    mediainfo_attr!(text_format_with_hint_list, "Text_Format_WithHint_List");
    mediainfo_attr!(text_language_list, "Text_Language_List");
    mediainfo_attr!(other_format_list, "Other_Format_List");
    mediainfo_attr!(other_format_with_hint_list, "Other_Format_WithHint_List");
    mediainfo_attr!(other_language_list, "Other_Language_List");
    mediainfo_attr!(image_format_list, "Image_Format_List");
    mediainfo_attr!(image_format_with_hint_list, "Image_Format_WithHint_List");
    mediainfo_attr!(image_language_list, "Image_Language_List");
    mediainfo_attr!(menu_format_list, "Menu_Format_List");
    mediainfo_attr!(menu_format_with_hint_list, "Menu_Format_WithHint_List");
    mediainfo_attr!(menu_language_list, "Menu_Language_List");

    // File names and paths
    mediainfo_attr!(complete_name, "CompleteName");
    mediainfo_attr!(folder_name, "FolderName");
    mediainfo_attr!(file_name_extension, "FileNameExtension");
    mediainfo_attr!(file_name, "FileName");
    mediainfo_attr!(file_extension, "FileExtension");
    mediainfo_attr!(complete_name_last, "CompleteName_Last");
    mediainfo_attr!(folder_name_last, "FolderName_Last");
    mediainfo_attr!(file_name_extension_last, "FileNameExtension_Last");
    mediainfo_attr!(file_name_last, "FileName_Last");
    mediainfo_attr!(file_extension_last, "FileExtension_Last");

    // Format details
    mediainfo_attr!(format_extensions, "Format_Extensions");
    mediainfo_attr!(format_level, "Format_Level");
    mediainfo_attr!(internet_media_type, "InternetMediaType");
    mediainfo_attr!(codec_id_version, "CodecID_Version");
    mediainfo_attr!(codec_id_compatible, "CodecID_Compatible");
    mediainfo_attr!(interleaved, "Interleaved");

    // File size
    mediainfo_attr!(file_size, "FileSize");
    mediainfo_attr!(file_size_string, "FileSize/String");
    mediainfo_attr!(file_size_string1, "FileSize/String1");
    mediainfo_attr!(file_size_string2, "FileSize/String2");
    mediainfo_attr!(file_size_string3, "FileSize/String3");
    mediainfo_attr!(file_size_string4, "FileSize/String4");

    // Duration fields
    mediainfo_duration!(duration, "Duration");
    mediainfo_attr!(duration_string, "Duration/String");
    mediainfo_attr!(duration_string1, "Duration/String1");
    mediainfo_attr!(duration_string2, "Duration/String2");
    mediainfo_attr!(duration_string3, "Duration/String3");
    mediainfo_attr!(duration_string4, "Duration/String4");
    mediainfo_attr!(duration_string5, "Duration/String5");

    // Duration start/end
    mediainfo_i64!(duration_start, "Duration_Start");
    mediainfo_attr!(duration_start_string, "Duration_Start/String");
    mediainfo_attr!(duration_start_string1, "Duration_Start/String1");
    mediainfo_attr!(duration_start_string2, "Duration_Start/String2");
    mediainfo_attr!(duration_start_string3, "Duration_Start/String3");
    mediainfo_attr!(duration_start_string4, "Duration_Start/String4");
    mediainfo_attr!(duration_start_string5, "Duration_Start/String5");

    mediainfo_i64!(duration_end, "Duration_End");
    mediainfo_attr!(duration_end_string, "Duration_End/String");
    mediainfo_attr!(duration_end_string1, "Duration_End/String1");
    mediainfo_attr!(duration_end_string2, "Duration_End/String2");
    mediainfo_attr!(duration_end_string3, "Duration_End/String3");
    mediainfo_attr!(duration_end_string4, "Duration_End/String4");
    mediainfo_attr!(duration_end_string5, "Duration_End/String5");

    // Bit rate fields
    mediainfo_attr!(overall_bit_rate_mode, "OverallBitRate_Mode");
    mediainfo_attr!(overall_bit_rate_mode_string, "OverallBitRate_Mode/String");
    mediainfo_i64!(overall_bit_rate, "OverallBitRate");
    mediainfo_attr!(overall_bit_rate_string, "OverallBitRate/String");
    mediainfo_i64!(overall_bit_rate_minimum, "OverallBitRate_Minimum");
    mediainfo_attr!(
        overall_bit_rate_minimum_string,
        "OverallBitRate_Minimum/String"
    );
    mediainfo_i64!(overall_bit_rate_nominal, "OverallBitRate_Nominal");
    mediainfo_attr!(
        overall_bit_rate_nominal_string,
        "OverallBitRate_Nominal/String"
    );
    mediainfo_i64!(overall_bit_rate_maximum, "OverallBitRate_Maximum");
    mediainfo_attr!(
        overall_bit_rate_maximum_string,
        "OverallBitRate_Maximum/String"
    );

    // Frame rate
    mediainfo_i64!(frame_rate, "FrameRate");
    mediainfo_attr!(frame_rate_string, "FrameRate/String");
    mediainfo_i64!(frame_rate_num, "FrameRate_Num");
    mediainfo_i64!(frame_rate_den, "FrameRate_Den");
    mediainfo_i64!(frame_count, "FrameCount");

    // Delay
    mediainfo_i64!(delay, "Delay");
    mediainfo_attr!(delay_string, "Delay/String");
    mediainfo_attr!(delay_string1, "Delay/String1");
    mediainfo_attr!(delay_string2, "Delay/String2");
    mediainfo_attr!(delay_string3, "Delay/String3");
    mediainfo_attr!(delay_string4, "Delay/String4");
    mediainfo_attr!(delay_string5, "Delay/String5");
    mediainfo_attr!(delay_settings, "Delay_Settings");
    mediainfo_attr!(delay_drop_frame, "Delay_DropFrame");
    mediainfo_attr!(delay_source, "Delay_Source");
    mediainfo_attr!(delay_source_string, "Delay_Source/String");

    // Stream size
    mediainfo_i64!(stream_size, "StreamSize");
    mediainfo_attr!(stream_size_string, "StreamSize/String");
    mediainfo_attr!(stream_size_string1, "StreamSize/String1");
    mediainfo_attr!(stream_size_string2, "StreamSize/String2");
    mediainfo_attr!(stream_size_string3, "StreamSize/String3");
    mediainfo_attr!(stream_size_string4, "StreamSize/String4");
    mediainfo_attr!(stream_size_string5, "StreamSize/String5");
    mediainfo_attr!(stream_size_proportion, "StreamSize_Proportion");

    mediainfo_i64!(stream_size_demuxed, "StreamSize_Demuxed");
    mediainfo_attr!(stream_size_demuxed_string, "StreamSize_Demuxed/String");
    mediainfo_attr!(stream_size_demuxed_string1, "StreamSize_Demuxed/String1");
    mediainfo_attr!(stream_size_demuxed_string2, "StreamSize_Demuxed/String2");
    mediainfo_attr!(stream_size_demuxed_string3, "StreamSize_Demuxed/String3");
    mediainfo_attr!(stream_size_demuxed_string4, "StreamSize_Demuxed/String4");
    mediainfo_attr!(stream_size_demuxed_string5, "StreamSize_Demuxed/String5");

    // Header/data/footer sizes
    mediainfo_i64!(headersize, "HeaderSize");
    mediainfo_i64!(datasize, "DataSize");
    mediainfo_i64!(footersize, "FooterSize");

    // Streamable
    mediainfo_attr!(is_streamable, "IsStreamable");

    // Replay gain
    mediainfo_attr!(album_replay_gain_gain, "Album_ReplayGain_Gain");
    mediainfo_attr!(
        album_replay_gain_gain_string,
        "Album_ReplayGain_Gain/String"
    );
    mediainfo_attr!(album_replay_gain_peak, "Album_ReplayGain_Peak");

    // Encryption
    mediainfo_attr!(encryption, "Encryption");
    mediainfo_attr!(encryption_format, "Encryption_Format");
    mediainfo_attr!(encryption_length, "Encryption_Length");
    mediainfo_attr!(encryption_method, "Encryption_Method");
    mediainfo_attr!(encryption_mode, "Encryption_Mode");
    mediainfo_attr!(encryption_padding, "Encryption_Padding");
    mediainfo_attr!(
        encryption_initialization_vector,
        "Encryption_InitializationVector"
    );

    mediainfo_date!(mastered_date, "Mastered_Date");
    mediainfo_date!(last_modification_date, "File_Modified_Date");
    mediainfo_date!(encoded_date, "Encoded_Date");
    mediainfo_date!(tagged_date, "Tagged_Date");

    pub fn writing_application(&self) -> MediaInfoResult<String> {
        match self.encoded_application() {
            Ok(x) => Ok(x),
            Err(_) => self.encoded_application_string(),
        }
    }
}

/* VideoStream */
impl VideoStream {
    // Basic stream identification
    mediainfo_attr!(stream_id, "ID");

    // Format details
    mediainfo_attr!(format, "Format");
    mediainfo_attr!(format_info, "Format_Info");
    mediainfo_attr!(format_profile, "Format_Profile");
    mediainfo_attr!(format_version, "Format_Version");
    mediainfo_attr!(format_level, "Format_Level");
    mediainfo_attr!(format_tier, "Format_Tier");
    mediainfo_attr!(format_commercial, "Format_Commercial");

    // Format settings
    mediainfo_attr!(format_settings_cabac, "Format_Settings_CABAC");
    mediainfo_attr!(format_settings_cabac_string, "Format_Settings_CABAC/String");
    mediainfo_attr!(format_settings_reframes, "Format_Settings_ReFrames");
    mediainfo_attr!(
        format_settings_reframes_string,
        "Format_Settings_ReFrames/String"
    );
    mediainfo_i64!(format_settings_ref_frames, "Format_Settings_RefFrames");
    mediainfo_attr!(
        format_settings_ref_frames_string,
        "Format_Settings_RefFrames/String"
    );
    mediainfo_attr!(format_settings_matrix, "Format_Settings_Matrix");
    mediainfo_attr!(
        format_settings_matrix_string,
        "Format_Settings_Matrix/String"
    );
    mediainfo_attr!(format_settings_matrix_data, "Format_Settings_Matrix_Data");
    mediainfo_attr!(format_settings_gop, "Format_Settings_GOP");
    mediainfo_attr!(format_settings_bvop, "Format_Settings_BVOP");
    mediainfo_attr!(format_settings_bvop_string, "Format_Settings_BVOP/String");
    mediainfo_attr!(format_settings_qpel, "Format_Settings_QPel");
    mediainfo_attr!(format_settings_qpel_string, "Format_Settings_QPel/String");
    mediainfo_i64!(format_settings_gmc, "Format_Settings_GMC");
    mediainfo_attr!(format_settings_gmc_string, "Format_Settings_GMC/String");
    mediainfo_attr!(format_settings_pulldown, "Format_Settings_Pulldown");
    mediainfo_attr!(format_settings_endianness, "Format_Settings_Endianness");
    mediainfo_attr!(format_settings_packing, "Format_Settings_Packing");
    mediainfo_attr!(format_settings_frame_mode, "Format_Settings_FrameMode");
    mediainfo_attr!(
        format_settings_picture_structure,
        "Format_Settings_PictureStructure"
    );
    mediainfo_attr!(format_settings_wrapping, "Format_Settings_Wrapping");
    mediainfo_i64!(format_settings_slice_count, "Format_Settings_SliceCount");
    mediainfo_attr!(
        format_settings_slice_count_string,
        "Format_Settings_SliceCount/String"
    );

    // Codec information
    mediainfo_attr!(codec_id, "CodecID");
    mediainfo_attr!(codec_info, "CodecID/Info");
    mediainfo_attr!(codec, "Codec");

    // Multiview
    mediainfo_attr!(multiview_base_profile, "MultiView_BaseProfile");
    mediainfo_attr!(multiview_count, "MultiView_Count");
    mediainfo_attr!(multiview_layout, "MultiView_Layout");

    // HDR format
    mediainfo_attr!(hdr_format, "HDR_Format");
    mediainfo_attr!(hdr_format_string, "HDR_Format/String");
    mediainfo_attr!(hdr_format_commercial, "HDR_Format_Commercial");
    mediainfo_attr!(hdr_format_version, "HDR_Format_Version");
    mediainfo_attr!(hdr_format_profile, "HDR_Format_Profile");
    mediainfo_attr!(hdr_format_level, "HDR_Format_Level");
    mediainfo_attr!(hdr_format_settings, "HDR_Format_Settings");
    mediainfo_attr!(hdr_format_compression, "HDR_Format_Compression");
    mediainfo_attr!(hdr_format_compatibility, "HDR_Format_Compatibility");

    // Internet media type
    mediainfo_attr!(internet_media_type, "InternetMediaType");

    // Muxing mode
    mediainfo_attr!(muxing_mode, "MuxingMode");

    // Duration fields
    mediainfo_duration!(duration, "Duration");
    mediainfo_attr!(duration_string, "Duration/String");
    mediainfo_attr!(duration_string1, "Duration/String1");
    mediainfo_attr!(duration_string2, "Duration/String2");
    mediainfo_attr!(duration_string3, "Duration/String3");
    mediainfo_attr!(duration_string4, "Duration/String4");
    mediainfo_attr!(duration_string5, "Duration/String5");

    mediainfo_i64!(duration_first_frame, "Duration_FirstFrame");
    mediainfo_attr!(duration_first_frame_string, "Duration_FirstFrame/String");
    mediainfo_attr!(duration_first_frame_string1, "Duration_FirstFrame/String1");
    mediainfo_attr!(duration_first_frame_string2, "Duration_FirstFrame/String2");
    mediainfo_attr!(duration_first_frame_string3, "Duration_FirstFrame/String3");
    mediainfo_attr!(duration_first_frame_string4, "Duration_FirstFrame/String4");
    mediainfo_attr!(duration_first_frame_string5, "Duration_FirstFrame/String5");

    mediainfo_i64!(duration_last_frame, "Duration_LastFrame");
    mediainfo_attr!(duration_last_frame_string, "Duration_LastFrame/String");
    mediainfo_attr!(duration_last_frame_string1, "Duration_LastFrame/String1");
    mediainfo_attr!(duration_last_frame_string2, "Duration_LastFrame/String2");
    mediainfo_attr!(duration_last_frame_string3, "Duration_LastFrame/String3");
    mediainfo_attr!(duration_last_frame_string4, "Duration_LastFrame/String4");
    mediainfo_attr!(duration_last_frame_string5, "Duration_LastFrame/String5");

    mediainfo_i64!(source_duration, "Source_Duration");
    mediainfo_attr!(source_duration_string, "Source_Duration/String");
    mediainfo_attr!(source_duration_string1, "Source_Duration/String1");
    mediainfo_attr!(source_duration_string2, "Source_Duration/String2");
    mediainfo_attr!(source_duration_string3, "Source_Duration/String3");
    mediainfo_attr!(source_duration_string4, "Source_Duration/String4");
    mediainfo_attr!(source_duration_string5, "Source_Duration/String5");

    mediainfo_i64!(source_duration_first_frame, "Source_Duration_FirstFrame");
    mediainfo_attr!(
        source_duration_first_frame_string,
        "Source_Duration_FirstFrame/String"
    );
    mediainfo_attr!(
        source_duration_first_frame_string1,
        "Source_Duration_FirstFrame/String1"
    );
    mediainfo_attr!(
        source_duration_first_frame_string2,
        "Source_Duration_FirstFrame/String2"
    );
    mediainfo_attr!(
        source_duration_first_frame_string3,
        "Source_Duration_FirstFrame/String3"
    );
    mediainfo_attr!(
        source_duration_first_frame_string4,
        "Source_Duration_FirstFrame/String4"
    );
    mediainfo_attr!(
        source_duration_first_frame_string5,
        "Source_Duration_FirstFrame/String5"
    );

    mediainfo_i64!(source_duration_last_frame, "Source_Duration_LastFrame");
    mediainfo_attr!(
        source_duration_last_frame_string,
        "Source_Duration_LastFrame/String"
    );
    mediainfo_attr!(
        source_duration_last_frame_string1,
        "Source_Duration_LastFrame/String1"
    );
    mediainfo_attr!(
        source_duration_last_frame_string2,
        "Source_Duration_LastFrame/String2"
    );
    mediainfo_attr!(
        source_duration_last_frame_string3,
        "Source_Duration_LastFrame/String3"
    );
    mediainfo_attr!(
        source_duration_last_frame_string4,
        "Source_Duration_LastFrame/String4"
    );
    mediainfo_attr!(
        source_duration_last_frame_string5,
        "Source_Duration_LastFrame/String5"
    );

    // Bit rate fields
    mediainfo_attr!(bit_rate_mode, "BitRate_Mode");
    mediainfo_attr!(bit_rate_mode_string, "BitRate_Mode/String");
    mediainfo_attr!(bit_rate, "BitRate");
    mediainfo_attr!(bit_rate_string, "BitRate/String");
    mediainfo_i64!(bit_rate_minimum, "BitRate_Minimum");
    mediainfo_attr!(bit_rate_minimum_string, "BitRate_Minimum/String");
    mediainfo_attr!(nominal_bit_rate, "BitRate_Nominal");
    mediainfo_attr!(bit_rate_nominal_string, "BitRate_Nominal/String");
    mediainfo_i64!(bit_rate_maximum, "BitRate_Maximum");
    mediainfo_attr!(bit_rate_maximum_string, "BitRate_Maximum/String");
    mediainfo_i64!(bit_rate_encoded, "BitRate_Encoded");
    mediainfo_attr!(bit_rate_encoded_string, "BitRate_Encoded/String");

    pub fn cbr(&self) -> bool {
        match self.bit_rate_mode() {
            Ok(x) => x == "Constant",
            Err(_) => false,
        }
    }

    pub fn vbr(&self) -> bool {
        !self.cbr()
    }

    // Dimensions
    mediainfo_i64!(width, "Width");
    mediainfo_attr!(width_string, "Width/String");
    mediainfo_i64!(width_offset, "Width_Offset");
    mediainfo_attr!(width_offset_string, "Width_Offset/String");
    mediainfo_i64!(width_original, "Width_Original");
    mediainfo_attr!(width_original_string, "Width_Original/String");
    mediainfo_i64!(width_clean_aperture, "Width_CleanAperture");
    mediainfo_attr!(width_clean_aperture_string, "Width_CleanAperture/String");

    mediainfo_i64!(height, "Height");
    mediainfo_attr!(height_string, "Height/String");
    mediainfo_i64!(height_offset, "Height_Offset");
    mediainfo_attr!(height_offset_string, "Height_Offset/String");
    mediainfo_i64!(height_original, "Height_Original");
    mediainfo_attr!(height_original_string, "Height_Original/String");
    mediainfo_i64!(height_clean_aperture, "Height_CleanAperture");
    mediainfo_attr!(height_clean_aperture_string, "Height_CleanAperture/String");

    mediainfo_i64!(stored_width, "Stored_Width");
    mediainfo_i64!(stored_height, "Stored_Height");
    mediainfo_i64!(sampled_width, "Sampled_Width");
    mediainfo_i64!(sampled_height, "Sampled_Height");

    // Aspect ratios
    mediainfo_attr!(pixel_aspect_ratio, "PixelAspectRatio");
    mediainfo_attr!(pixel_aspect_ratio_string, "PixelAspectRatio/String");
    mediainfo_attr!(pixel_aspect_ratio_original, "PixelAspectRatio_Original");
    mediainfo_attr!(
        pixel_aspect_ratio_original_string,
        "PixelAspectRatio_Original/String"
    );
    mediainfo_attr!(
        pixel_aspect_ratio_clean_aperture,
        "PixelAspectRatio_CleanAperture"
    );
    mediainfo_attr!(
        pixel_aspect_ratio_clean_aperture_string,
        "PixelAspectRatio_CleanAperture/String"
    );

    mediainfo_attr!(display_aspect_ratio, "DisplayAspectRatio");
    mediainfo_attr!(display_aspect_ratio_string, "DisplayAspectRatio/String");
    mediainfo_attr!(display_aspect_ratio_original, "DisplayAspectRatio_Original");
    mediainfo_attr!(
        display_aspect_ratio_original_string,
        "DisplayAspectRatio_Original/String"
    );
    mediainfo_attr!(
        display_aspect_ratio_clean_aperture,
        "DisplayAspectRatio_CleanAperture"
    );
    mediainfo_attr!(
        display_aspect_ratio_clean_aperture_string,
        "DisplayAspectRatio_CleanAperture/String"
    );

    // Active format description
    mediainfo_attr!(active_format_description, "ActiveFormatDescription");
    mediainfo_attr!(
        active_format_description_string,
        "ActiveFormatDescription/String"
    );
    mediainfo_attr!(
        active_format_description_muxing_mode,
        "ActiveFormatDescription_MuxingMode"
    );

    // Active dimensions
    mediainfo_i64!(active_width, "Active_Width");
    mediainfo_attr!(active_width_string, "Active_Width/String");
    mediainfo_i64!(active_height, "Active_Height");
    mediainfo_attr!(active_height_string, "Active_Height/String");
    mediainfo_attr!(active_display_aspect_ratio, "Active_DisplayAspectRatio");
    mediainfo_attr!(
        active_display_aspect_ratio_string,
        "Active_DisplayAspectRatio/String"
    );

    // Rotation
    mediainfo_attr!(rotation, "Rotation");
    mediainfo_attr!(rotation_string, "Rotation/String");

    // Frame rate
    mediainfo_attr!(frame_rate_mode, "FrameRate_Mode");
    mediainfo_attr!(frame_rate_mode_string, "FrameRate_Mode/String");
    mediainfo_attr!(frame_rate_mode_original, "FrameRate_Mode_Original");
    mediainfo_attr!(
        frame_rate_mode_original_string,
        "FrameRate_Mode_Original/String"
    );

    mediainfo_attr!(frame_rate, "FrameRate");
    mediainfo_attr!(frame_rate_string, "FrameRate/String");
    mediainfo_i64!(frame_rate_num, "FrameRate_Num");
    mediainfo_i64!(frame_rate_den, "FrameRate_Den");
    mediainfo_attr!(minimum_frame_rate, "FrameRate_Minimum");
    mediainfo_attr!(frame_rate_minimum_string, "FrameRate_Minimum/String");
    mediainfo_attr!(nominal_frame_rate, "FrameRate_Nominal");
    mediainfo_attr!(frame_rate_nominal_string, "FrameRate_Nominal/String");
    mediainfo_attr!(maximum_frame_rate, "FrameRate_Maximum");
    mediainfo_attr!(frame_rate_maximum_string, "FrameRate_Maximum/String");

    mediainfo_attr!(frame_rate_original, "FrameRate_Original");
    mediainfo_attr!(frame_rate_original_string, "FrameRate_Original/String");
    mediainfo_i64!(frame_rate_original_num, "FrameRate_Original_Num");
    mediainfo_i64!(frame_rate_original_den, "FrameRate_Original_Den");

    mediainfo_attr!(frame_rate_real, "FrameRate_Real");
    mediainfo_attr!(frame_rate_real_string, "FrameRate_Real/String");

    mediainfo_i64!(frame_count, "FrameCount");
    mediainfo_i64!(source_frame_count, "Source_FrameCount");

    // Standard
    mediainfo_attr!(standard, "Standard");

    // Color
    mediainfo_attr!(colorspace, "ColorSpace");
    mediainfo_attr!(chroma_subsampling, "ChromaSubsampling");
    mediainfo_attr!(chroma_subsampling_string, "ChromaSubsampling/String");
    mediainfo_attr!(chroma_subsampling_position, "ChromaSubsampling_Position");

    mediainfo_i64!(bitdepth, "BitDepth");
    mediainfo_attr!(bit_depth_string, "BitDepth/String");

    // Scan type
    mediainfo_attr!(scan_type, "ScanType");
    mediainfo_attr!(scan_type_string, "ScanType/String");
    mediainfo_attr!(scan_type_original, "ScanType_Original");
    mediainfo_attr!(scan_type_original_string, "ScanType_Original/String");
    mediainfo_attr!(scan_type_store_method, "ScanType_StoreMethod");
    mediainfo_attr!(
        scan_type_store_method_fields_per_block,
        "ScanType_StoreMethod_FieldsPerBlock"
    );
    mediainfo_attr!(scan_type_store_method_string, "ScanType_StoreMethod/String");

    mediainfo_attr!(scan_order, "ScanOrder");
    mediainfo_attr!(scan_order_string, "ScanOrder/String");
    mediainfo_attr!(scan_order_stored, "ScanOrder_Stored");
    mediainfo_attr!(scan_order_stored_string, "ScanOrder_Stored/String");
    mediainfo_attr!(
        scan_order_stored_displayed_inverted,
        "ScanOrder_StoredDisplayedInverted"
    );
    mediainfo_attr!(scan_order_original, "ScanOrder_Original");
    mediainfo_attr!(scan_order_original_string, "ScanOrder_Original/String");

    pub fn interlaced(&self) -> bool {
        match self.scan_type() {
            Ok(x) => x == "Interlaced",
            Err(_) => false,
        }
    }

    pub fn progressive(&self) -> bool {
        !self.interlaced()
    }

    // Compression
    mediainfo_attr!(compression_mode, "Compression_Mode");
    mediainfo_attr!(compression_mode_string, "Compression_Mode/String");
    mediainfo_attr!(compression_ratio, "Compression_Ratio");

    // Bits per pixel
    mediainfo_attr!(bits_pixel_frame, "Bits-(Pixel*Frame)");

    // Resolution
    mediainfo_i64!(resolution, "Resolution");

    pub fn frame_size(&self) -> MediaInfoResult<String> {
        let height = self.height()?;
        let width = self.width()?;

        Ok(format!("{}x{}", width, height))
    }

    // Stream size
    mediainfo_attr!(stream_size, "StreamSize");
    mediainfo_attr!(stream_size_string, "StreamSize/String");
    mediainfo_attr!(stream_size_string1, "StreamSize/String1");
    mediainfo_attr!(stream_size_string2, "StreamSize/String2");
    mediainfo_attr!(stream_size_string3, "StreamSize/String3");
    mediainfo_attr!(stream_size_string4, "StreamSize/String4");
    mediainfo_attr!(stream_size_string5, "StreamSize/String5");
    mediainfo_attr!(stream_size_proportion, "StreamSize_Proportion");

    mediainfo_date!(encoded_date, "Encoded_Date");
    mediainfo_date!(tagged_date, "Tagged_Date");
}

/* AudioStream */
impl AudioStream {
    // Basic stream identification
    mediainfo_attr!(stream_id, "ID");

    // Format details
    mediainfo_attr!(format, "Format");
    mediainfo_attr!(format_string, "Format/String");
    mediainfo_attr!(format_info, "Format_Info");
    mediainfo_attr!(format_url, "Format_Url");
    mediainfo_attr!(format_commercial, "Format_Commercial");
    mediainfo_attr!(format_commercial_if_any, "Format_Commercial_IfAny");
    mediainfo_attr!(format_version, "Format_Version");
    mediainfo_attr!(format_profile, "Format_Profile");
    mediainfo_attr!(format_compression, "Format_Compression");
    mediainfo_attr!(format_settings, "Format_Settings");
    mediainfo_attr!(format_additional_features, "Format_AdditionalFeatures");

    // Format settings
    mediainfo_attr!(format_level, "Format_Level");
    mediainfo_attr!(format_settings_sbr, "Format_Settings_SBR");
    mediainfo_attr!(format_settings_sbr_string, "Format_Settings_SBR/String");
    mediainfo_attr!(format_settings_ps, "Format_Settings_PS");
    mediainfo_attr!(format_settings_ps_string, "Format_Settings_PS/String");
    mediainfo_attr!(format_settings_mode, "Format_Settings_Mode");
    mediainfo_attr!(
        format_settings_mode_extension,
        "Format_Settings_ModeExtension"
    );
    mediainfo_attr!(format_settings_emphasis, "Format_Settings_Emphasis");
    mediainfo_attr!(format_settings_floor, "Format_Settings_Floor");
    mediainfo_attr!(format_settings_firm, "Format_Settings_Firm");
    mediainfo_attr!(format_settings_endianness, "Format_Settings_Endianness");
    mediainfo_attr!(format_settings_sign, "Format_Settings_Sign");
    mediainfo_attr!(format_settings_law, "Format_Settings_Law");
    mediainfo_attr!(format_settings_itu, "Format_Settings_ITU");
    mediainfo_attr!(format_settings_wrapping, "Format_Settings_Wrapping");

    // Matrix format
    mediainfo_attr!(matrix_format, "Matrix_Format");

    // Codec information
    mediainfo_attr!(codec_id, "CodecID");
    mediainfo_attr!(codec_id_string, "CodecID/String");
    mediainfo_attr!(codec_info, "CodecID/Info");
    mediainfo_attr!(codec_id_hint, "CodecID/Hint");
    mediainfo_attr!(codec_id_url, "CodecID/Url");
    mediainfo_attr!(codec_id_description, "CodecID_Description");

    // Internet media type
    mediainfo_attr!(internet_media_type, "InternetMediaType");

    // Muxing mode
    mediainfo_attr!(muxing_mode, "MuxingMode");
    mediainfo_attr!(muxing_mode_more_info, "MuxingMode_MoreInfo");

    // Duration fields
    mediainfo_duration!(duration, "Duration");
    mediainfo_attr!(duration_string, "Duration/String");
    mediainfo_attr!(duration_string1, "Duration/String1");
    mediainfo_attr!(duration_string2, "Duration/String2");
    mediainfo_attr!(duration_string3, "Duration/String3");
    mediainfo_attr!(duration_string4, "Duration/String4");
    mediainfo_attr!(duration_string5, "Duration/String5");

    mediainfo_i64!(duration_first_frame, "Duration_FirstFrame");
    mediainfo_attr!(duration_first_frame_string, "Duration_FirstFrame/String");
    mediainfo_attr!(duration_first_frame_string1, "Duration_FirstFrame/String1");
    mediainfo_attr!(duration_first_frame_string2, "Duration_FirstFrame/String2");
    mediainfo_attr!(duration_first_frame_string3, "Duration_FirstFrame/String3");
    mediainfo_attr!(duration_first_frame_string4, "Duration_FirstFrame/String4");
    mediainfo_attr!(duration_first_frame_string5, "Duration_FirstFrame/String5");

    mediainfo_i64!(duration_last_frame, "Duration_LastFrame");
    mediainfo_attr!(duration_last_frame_string, "Duration_LastFrame/String");
    mediainfo_attr!(duration_last_frame_string1, "Duration_LastFrame/String1");
    mediainfo_attr!(duration_last_frame_string2, "Duration_LastFrame/String2");
    mediainfo_attr!(duration_last_frame_string3, "Duration_LastFrame/String3");
    mediainfo_attr!(duration_last_frame_string4, "Duration_LastFrame/String4");
    mediainfo_attr!(duration_last_frame_string5, "Duration_LastFrame/String5");

    mediainfo_i64!(source_duration, "Source_Duration");
    mediainfo_attr!(source_duration_string, "Source_Duration/String");
    mediainfo_attr!(source_duration_string1, "Source_Duration/String1");
    mediainfo_attr!(source_duration_string2, "Source_Duration/String2");
    mediainfo_attr!(source_duration_string3, "Source_Duration/String3");
    mediainfo_attr!(source_duration_string4, "Source_Duration/String4");
    mediainfo_attr!(source_duration_string5, "Source_Duration/String5");

    mediainfo_i64!(source_duration_first_frame, "Source_Duration_FirstFrame");
    mediainfo_attr!(
        source_duration_first_frame_string,
        "Source_Duration_FirstFrame/String"
    );
    mediainfo_attr!(
        source_duration_first_frame_string1,
        "Source_Duration_FirstFrame/String1"
    );
    mediainfo_attr!(
        source_duration_first_frame_string2,
        "Source_Duration_FirstFrame/String2"
    );
    mediainfo_attr!(
        source_duration_first_frame_string3,
        "Source_Duration_FirstFrame/String3"
    );
    mediainfo_attr!(
        source_duration_first_frame_string4,
        "Source_Duration_FirstFrame/String4"
    );
    mediainfo_attr!(
        source_duration_first_frame_string5,
        "Source_Duration_FirstFrame/String5"
    );

    mediainfo_i64!(source_duration_last_frame, "Source_Duration_LastFrame");
    mediainfo_attr!(
        source_duration_last_frame_string,
        "Source_Duration_LastFrame/String"
    );
    mediainfo_attr!(
        source_duration_last_frame_string1,
        "Source_Duration_LastFrame/String1"
    );
    mediainfo_attr!(
        source_duration_last_frame_string2,
        "Source_Duration_LastFrame/String2"
    );
    mediainfo_attr!(
        source_duration_last_frame_string3,
        "Source_Duration_LastFrame/String3"
    );
    mediainfo_attr!(
        source_duration_last_frame_string4,
        "Source_Duration_LastFrame/String4"
    );
    mediainfo_attr!(
        source_duration_last_frame_string5,
        "Source_Duration_LastFrame/String5"
    );

    // Bit rate fields
    mediainfo_attr!(bit_rate_mode, "BitRate_Mode");
    mediainfo_attr!(bit_rate_mode_string, "BitRate_Mode/String");
    mediainfo_attr!(bit_rate, "BitRate");
    mediainfo_attr!(bit_rate_string, "BitRate/String");
    mediainfo_i64!(bit_rate_minimum, "BitRate_Minimum");
    mediainfo_attr!(bit_rate_minimum_string, "BitRate_Minimum/String");
    mediainfo_i64!(bit_rate_nominal, "BitRate_Nominal");
    mediainfo_attr!(bit_rate_nominal_string, "BitRate_Nominal/String");
    mediainfo_i64!(bit_rate_maximum, "BitRate_Maximum");
    mediainfo_attr!(bit_rate_maximum_string, "BitRate_Maximum/String");
    mediainfo_i64!(bit_rate_encoded, "BitRate_Encoded");
    mediainfo_attr!(bit_rate_encoded_string, "BitRate_Encoded/String");

    // Channel information
    mediainfo_i64!(channels, "Channels");
    mediainfo_attr!(channels_string, "Channels/String");
    mediainfo_i64!(channels_original, "Channels_Original");
    mediainfo_attr!(channels_original_string, "Channels_Original/String");
    mediainfo_i64!(matrix_channels, "Matrix_Channels");
    mediainfo_attr!(matrix_channels_string, "Matrix_Channels/String");

    // Channel positions
    mediainfo_attr!(channel_positions, "ChannelPositions");
    mediainfo_attr!(channel_positions_original, "ChannelPositions_Original");
    mediainfo_attr!(channel_positions_string2, "ChannelPositions/String2");
    mediainfo_attr!(
        channel_positions_original_string2,
        "ChannelPositions_Original/String2"
    );
    mediainfo_attr!(matrix_channel_positions, "Matrix_ChannelPositions");
    mediainfo_attr!(
        matrix_channel_positions_string2,
        "Matrix_ChannelPositions/String2"
    );

    // Channel layout
    mediainfo_attr!(channel_layout, "ChannelLayout");
    mediainfo_attr!(channel_layout_original, "ChannelLayout_Original");
    mediainfo_attr!(channel_layout_id, "ChannelLayoutID");

    // Sampling
    mediainfo_i64!(samples_per_frame, "SamplesPerFrame");
    mediainfo_i64!(sampling_rate, "SamplingRate");
    mediainfo_attr!(sampling_rate_string, "SamplingRate/String");
    mediainfo_attr!(sampling_count, "SamplingCount");
    mediainfo_i64!(source_sampling_count, "Source_SamplingCount");

    // Frame rate
    mediainfo_attr!(frame_rate, "FrameRate");
    mediainfo_attr!(frame_rate_string, "FrameRate/String");
    mediainfo_i64!(frame_rate_num, "FrameRate_Num");
    mediainfo_i64!(frame_rate_den, "FrameRate_Den");
    mediainfo_i64!(frame_count, "FrameCount");
    mediainfo_i64!(source_frame_count, "Source_FrameCount");

    // Bit depth
    mediainfo_i64!(bit_depth, "BitDepth");
    mediainfo_attr!(bit_depth_string, "BitDepth/String");
    mediainfo_i64!(bit_depth_detected, "BitDepth_Detected");
    mediainfo_attr!(bit_depth_detected_string, "BitDepth_Detected/String");
    mediainfo_i64!(bit_depth_stored, "BitDepth_Stored");
    mediainfo_attr!(bit_depth_stored_string, "BitDepth_Stored/String");

    // Resolution
    mediainfo_i64!(resolution, "Resolution");

    // Compression
    mediainfo_attr!(compression_mode, "Compression_Mode");
    mediainfo_attr!(compression_mode_string, "Compression_Mode/String");
    mediainfo_attr!(compression_ratio, "Compression_Ratio");

    // Delay
    mediainfo_i64!(delay, "Delay");
    mediainfo_attr!(delay_string, "Delay/String");
    mediainfo_attr!(delay_string1, "Delay/String1");
    mediainfo_attr!(delay_string2, "Delay/String2");
    mediainfo_attr!(delay_string3, "Delay/String3");
    mediainfo_attr!(delay_string4, "Delay/String4");
    mediainfo_attr!(delay_string5, "Delay/String5");
    mediainfo_attr!(delay_settings, "Delay_Settings");
    mediainfo_attr!(delay_drop_frame, "Delay_DropFrame");
    mediainfo_attr!(delay_source, "Delay_Source");
    mediainfo_attr!(delay_source_string, "Delay_Source/String");

    mediainfo_i64!(delay_original, "Delay_Original");
    mediainfo_attr!(delay_original_string, "Delay_Original/String");
    mediainfo_attr!(delay_original_string1, "Delay_Original/String1");
    mediainfo_attr!(delay_original_string2, "Delay_Original/String2");
    mediainfo_attr!(delay_original_string3, "Delay_Original/String3");
    mediainfo_attr!(delay_original_string4, "Delay_Original/String4");
    mediainfo_attr!(delay_original_string5, "Delay_Original/String5");
    mediainfo_attr!(delay_original_settings, "Delay_Original_Settings");
    mediainfo_attr!(delay_original_drop_frame, "Delay_Original_DropFrame");
    mediainfo_attr!(delay_original_source, "Delay_Original_Source");

    mediainfo_i64!(video_delay, "Video_Delay");
    mediainfo_attr!(video_delay_string, "Video_Delay/String");
    mediainfo_attr!(video_delay_string1, "Video_Delay/String1");
    mediainfo_attr!(video_delay_string2, "Video_Delay/String2");
    mediainfo_attr!(video_delay_string3, "Video_Delay/String3");
    mediainfo_attr!(video_delay_string4, "Video_Delay/String4");
    mediainfo_attr!(video_delay_string5, "Video_Delay/String5");

    // Time code
    mediainfo_attr!(time_code_first_frame, "TimeCode_FirstFrame");
    mediainfo_attr!(time_code_last_frame, "TimeCode_LastFrame");
    mediainfo_attr!(time_code_drop_frame, "TimeCode_DropFrame");
    mediainfo_attr!(time_code_settings, "TimeCode_Settings");
    mediainfo_attr!(time_code_source, "TimeCode_Source");

    // Replay gain
    mediainfo_attr!(replay_gain_gain, "ReplayGain_Gain");
    mediainfo_attr!(replay_gain_gain_string, "ReplayGain_Gain/String");
    mediainfo_attr!(replay_gain_peak, "ReplayGain_Peak");

    // Stream size
    mediainfo_attr!(stream_size, "StreamSize");
    mediainfo_attr!(stream_size_string, "StreamSize/String");
    mediainfo_attr!(stream_size_string1, "StreamSize/String1");
    mediainfo_attr!(stream_size_string2, "StreamSize/String2");
    mediainfo_attr!(stream_size_string3, "StreamSize/String3");
    mediainfo_attr!(stream_size_string4, "StreamSize/String4");
    mediainfo_attr!(stream_size_string5, "StreamSize/String5");
    mediainfo_attr!(stream_size_proportion, "StreamSize_Proportion");

    mediainfo_i64!(stream_size_demuxed, "StreamSize_Demuxed");
    mediainfo_attr!(stream_size_demuxed_string, "StreamSize_Demuxed/String");
    mediainfo_attr!(stream_size_demuxed_string1, "StreamSize_Demuxed/String1");
    mediainfo_attr!(stream_size_demuxed_string2, "StreamSize_Demuxed/String2");
    mediainfo_attr!(stream_size_demuxed_string3, "StreamSize_Demuxed/String3");
    mediainfo_attr!(stream_size_demuxed_string4, "StreamSize_Demuxed/String4");
    mediainfo_attr!(stream_size_demuxed_string5, "StreamSize_Demuxed/String5");

    mediainfo_i64!(source_stream_size, "Source_StreamSize");
    mediainfo_attr!(source_stream_size_string, "Source_StreamSize/String");
    mediainfo_attr!(source_stream_size_string1, "Source_StreamSize/String1");
    mediainfo_attr!(source_stream_size_string2, "Source_StreamSize/String2");
    mediainfo_attr!(source_stream_size_string3, "Source_StreamSize/String3");
    mediainfo_attr!(source_stream_size_string4, "Source_StreamSize/String4");
    mediainfo_attr!(source_stream_size_string5, "Source_StreamSize/String5");
    mediainfo_attr!(
        source_stream_size_proportion,
        "Source_StreamSize_Proportion"
    );

    mediainfo_i64!(stream_size_encoded, "StreamSize_Encoded");
    mediainfo_attr!(stream_size_encoded_string, "StreamSize_Encoded/String");
    mediainfo_attr!(stream_size_encoded_string1, "StreamSize_Encoded/String1");
    mediainfo_attr!(stream_size_encoded_string2, "StreamSize_Encoded/String2");
    mediainfo_attr!(stream_size_encoded_string3, "StreamSize_Encoded/String3");
    mediainfo_attr!(stream_size_encoded_string4, "StreamSize_Encoded/String4");
    mediainfo_attr!(stream_size_encoded_string5, "StreamSize_Encoded/String5");
    mediainfo_attr!(
        stream_size_encoded_proportion,
        "StreamSize_Encoded_Proportion"
    );

    mediainfo_i64!(source_stream_size_encoded, "Source_StreamSize_Encoded");
    mediainfo_attr!(
        source_stream_size_encoded_string,
        "Source_StreamSize_Encoded/String"
    );
    mediainfo_attr!(
        source_stream_size_encoded_string1,
        "Source_StreamSize_Encoded/String1"
    );
    mediainfo_attr!(
        source_stream_size_encoded_string2,
        "Source_StreamSize_Encoded/String2"
    );
    mediainfo_attr!(
        source_stream_size_encoded_string3,
        "Source_StreamSize_Encoded/String3"
    );
    mediainfo_attr!(
        source_stream_size_encoded_string4,
        "Source_StreamSize_Encoded/String4"
    );
    mediainfo_attr!(
        source_stream_size_encoded_string5,
        "Source_StreamSize_Encoded/String5"
    );
    mediainfo_attr!(
        source_stream_size_encoded_proportion,
        "Source_StreamSize_Encoded_Proportion"
    );

    // Alignment
    mediainfo_attr!(alignment, "Alignment");
    mediainfo_attr!(alignment_string, "Alignment/String");

    // Interleave
    mediainfo_i64!(interleave_video_frames, "Interleave_VideoFrames");
    mediainfo_i64!(interleave_duration, "Interleave_Duration");
    mediainfo_attr!(interleave_duration_string, "Interleave_Duration/String");
    mediainfo_i64!(interleave_preload, "Interleave_Preload");
    mediainfo_attr!(interleave_preload_string, "Interleave_Preload/String");

    // Title
    mediainfo_attr!(title, "Title");

    // Technical fields
    mediainfo_attr!(encoded_application, "Encoded_Application");
    mediainfo_attr!(encoded_application_string, "Encoded_Application/String");
    mediainfo_attr!(
        encoded_application_company_name,
        "Encoded_Application_CompanyName"
    );
    mediainfo_attr!(encoded_application_name, "Encoded_Application_Name");
    mediainfo_attr!(encoded_application_version, "Encoded_Application_Version");
    mediainfo_attr!(encoded_application_url, "Encoded_Application_Url");

    mediainfo_attr!(encoded_library, "Encoded_Library");
    mediainfo_attr!(encoded_library_string, "Encoded_Library/String");
    mediainfo_attr!(encoded_library_company_name, "Encoded_Library_CompanyName");
    mediainfo_attr!(encoded_library_name, "Encoded_Library_Name");
    mediainfo_attr!(encoded_library_version, "Encoded_Library_Version");
    mediainfo_attr!(encoded_library_date, "Encoded_Library_Date");
    mediainfo_attr!(encoded_library_settings, "Encoded_Library_Settings");
    mediainfo_attr!(encoded_operating_system, "Encoded_OperatingSystem");

    // Language
    mediainfo_attr!(language, "Language");
    mediainfo_attr!(language_string, "Language/String");
    mediainfo_attr!(language_string1, "Language/String1");
    mediainfo_attr!(language_string2, "Language/String2");
    mediainfo_attr!(language_string3, "Language/String3");
    mediainfo_attr!(language_string4, "Language/String4");
    mediainfo_attr!(language_more, "Language_More");

    // Service kind
    mediainfo_attr!(service_kind, "ServiceKind");
    mediainfo_attr!(service_kind_string, "ServiceKind/String");

    // Flags
    mediainfo_attr!(disabled, "Disabled");
    mediainfo_attr!(disabled_string, "Disabled/String");
    mediainfo_attr!(default, "Default");
    mediainfo_attr!(default_string, "Default/String");
    mediainfo_attr!(forced, "Forced");
    mediainfo_attr!(forced_string, "Forced/String");
    mediainfo_attr!(alternate_group, "AlternateGroup");
    mediainfo_attr!(alternate_group_string, "AlternateGroup/String");

    // Dates
    mediainfo_date!(encoded_date, "Encoded_Date");
    mediainfo_date!(tagged_date, "Tagged_Date");

    // Encryption
    mediainfo_attr!(encryption, "Encryption");

    pub fn stereo(&self) -> bool {
        match self.channels() {
            Ok(x) => x == 2,
            Err(_) => false,
        }
    }

    pub fn mono(&self) -> bool {
        match self.channels() {
            Ok(x) => x == 1,
            Err(_) => false,
        }
    }
}

impl ImageStream {
    // Title
    mediainfo_attr!(title, "Title");

    // HDR format
    mediainfo_attr!(hdr_format, "HDR_Format");
    mediainfo_attr!(hdr_format_string, "HDR_Format/String");
    mediainfo_attr!(hdr_format_commercial, "HDR_Format_Commercial");
    mediainfo_attr!(hdr_format_version, "HDR_Format_Version");
    mediainfo_attr!(hdr_format_profile, "HDR_Format_Profile");
    mediainfo_attr!(hdr_format_level, "HDR_Format_Level");
    mediainfo_attr!(hdr_format_settings, "HDR_Format_Settings");
    mediainfo_attr!(hdr_format_compatibility, "HDR_Format_Compatibility");

    // Format settings
    mediainfo_attr!(format_settings_endianness, "Format_Settings_Endianness");
    mediainfo_attr!(format_settings_packing, "Format_Settings_Packing");
    mediainfo_attr!(format_settings_wrapping, "Format_Settings_Wrapping");

    // Internet media type
    mediainfo_attr!(internet_media_type, "InternetMediaType");

    // Dimensions
    mediainfo_i64!(width, "Width");
    mediainfo_attr!(width_string, "Width/String");
    mediainfo_i64!(width_offset, "Width_Offset");
    mediainfo_attr!(width_offset_string, "Width_Offset/String");
    mediainfo_i64!(width_original, "Width_Original");
    mediainfo_attr!(width_original_string, "Width_Original/String");

    mediainfo_i64!(height, "Height");
    mediainfo_attr!(height_string, "Height/String");
    mediainfo_i64!(height_offset, "Height_Offset");
    mediainfo_attr!(height_offset_string, "Height_Offset/String");
    mediainfo_i64!(height_original, "Height_Original");
    mediainfo_attr!(height_original_string, "Height_Original/String");

    // Aspect ratios
    mediainfo_attr!(pixel_aspect_ratio, "PixelAspectRatio");
    mediainfo_attr!(pixel_aspect_ratio_string, "PixelAspectRatio/String");
    mediainfo_attr!(pixel_aspect_ratio_original, "PixelAspectRatio_Original");
    mediainfo_attr!(
        pixel_aspect_ratio_original_string,
        "PixelAspectRatio_Original/String"
    );

    mediainfo_attr!(display_aspect_ratio, "DisplayAspectRatio");
    mediainfo_attr!(display_aspect_ratio_string, "DisplayAspectRatio/String");
    mediainfo_attr!(display_aspect_ratio_original, "DisplayAspectRatio_Original");
    mediainfo_attr!(
        display_aspect_ratio_original_string,
        "DisplayAspectRatio_Original/String"
    );

    // Active dimensions
    mediainfo_i64!(active_width, "Active_Width");
    mediainfo_attr!(active_width_string, "Active_Width/String");
    mediainfo_i64!(active_height, "Active_Height");
    mediainfo_attr!(active_height_string, "Active_Height/String");
    mediainfo_attr!(active_display_aspect_ratio, "Active_DisplayAspectRatio");
    mediainfo_attr!(
        active_display_aspect_ratio_string,
        "Active_DisplayAspectRatio/String"
    );

    // Color
    mediainfo_attr!(color_space, "ColorSpace");
    mediainfo_attr!(chroma_subsampling, "ChromaSubsampling");
    mediainfo_i64!(bit_depth, "BitDepth");
    mediainfo_attr!(bit_depth_string, "BitDepth/String");

    // Compression
    mediainfo_attr!(compression_mode, "Compression_Mode");
    mediainfo_attr!(compression_mode_string, "Compression_Mode/String");
    mediainfo_attr!(compression_ratio, "Compression_Ratio");

    // Stream size
    mediainfo_i64!(stream_size, "StreamSize");
    mediainfo_attr!(stream_size_string, "StreamSize/String");
    mediainfo_attr!(stream_size_string1, "StreamSize/String1");
    mediainfo_attr!(stream_size_string2, "StreamSize/String2");
    mediainfo_attr!(stream_size_string3, "StreamSize/String3");
    mediainfo_attr!(stream_size_string4, "StreamSize/String4");
    mediainfo_attr!(stream_size_string5, "StreamSize/String5");
    mediainfo_attr!(stream_size_proportion, "StreamSize_Proportion");

    mediainfo_i64!(stream_size_demuxed, "StreamSize_Demuxed");
    mediainfo_attr!(stream_size_demuxed_string, "StreamSize_Demuxed/String");
    mediainfo_attr!(stream_size_demuxed_string1, "StreamSize_Demuxed/String1");
    mediainfo_attr!(stream_size_demuxed_string2, "StreamSize_Demuxed/String2");
    mediainfo_attr!(stream_size_demuxed_string3, "StreamSize_Demuxed/String3");
    mediainfo_attr!(stream_size_demuxed_string4, "StreamSize_Demuxed/String4");
    mediainfo_attr!(stream_size_demuxed_string5, "StreamSize_Demuxed/String5");

    // Technical fields
    mediainfo_attr!(encoded_library, "Encoded_Library");
    mediainfo_attr!(encoded_library_string, "Encoded_Library/String");
    mediainfo_attr!(encoded_library_name, "Encoded_Library_Name");
    mediainfo_attr!(encoded_library_version, "Encoded_Library_Version");
    mediainfo_attr!(encoded_library_date, "Encoded_Library_Date");
    mediainfo_attr!(encoded_library_settings, "Encoded_Library_Settings");

    // Language
    mediainfo_attr!(language, "Language");
    mediainfo_attr!(language_string, "Language/String");
    mediainfo_attr!(language_string1, "Language/String1");
    mediainfo_attr!(language_string2, "Language/String2");
    mediainfo_attr!(language_string3, "Language/String3");
    mediainfo_attr!(language_string4, "Language/String4");
    mediainfo_attr!(language_more, "Language_More");

    // Service kind
    mediainfo_attr!(service_kind, "ServiceKind");
    mediainfo_attr!(service_kind_string, "ServiceKind/String");

    // Flags
    mediainfo_attr!(disabled, "Disabled");
    mediainfo_attr!(disabled_string, "Disabled/String");
    mediainfo_attr!(default, "Default");
    mediainfo_attr!(default_string, "Default/String");
    mediainfo_attr!(forced, "Forced");
    mediainfo_attr!(forced_string, "Forced/String");
    mediainfo_attr!(alternate_group, "AlternateGroup");
    mediainfo_attr!(alternate_group_string, "AlternateGroup/String");

    // Summary
    mediainfo_attr!(summary, "Summary");

    // Dates
    mediainfo_date!(encoded_date, "Encoded_Date");
    mediainfo_date!(tagged_date, "Tagged_Date");

    // Encryption
    mediainfo_attr!(encryption, "Encryption");

    // Color description and characteristics (HDR/color science)
    mediainfo_attr!(colour_description_present, "colour_description_present");
    mediainfo_attr!(
        colour_description_present_source,
        "colour_description_present_Source"
    );
    mediainfo_attr!(
        colour_description_present_original,
        "colour_description_present_Original"
    );
    mediainfo_attr!(
        colour_description_present_original_source,
        "colour_description_present_Original_Source"
    );

    mediainfo_attr!(colour_range, "colour_range");
    mediainfo_attr!(colour_range_source, "colour_range_Source");
    mediainfo_attr!(colour_range_original, "colour_range_Original");
    mediainfo_attr!(colour_range_original_source, "colour_range_Original_Source");

    mediainfo_attr!(colour_primaries, "colour_primaries");
    mediainfo_attr!(colour_primaries_source, "colour_primaries_Source");
    mediainfo_attr!(colour_primaries_original, "colour_primaries_Original");
    mediainfo_attr!(
        colour_primaries_original_source,
        "colour_primaries_Original_Source"
    );

    mediainfo_attr!(transfer_characteristics, "transfer_characteristics");
    mediainfo_attr!(
        transfer_characteristics_source,
        "transfer_characteristics_Source"
    );
    mediainfo_attr!(
        transfer_characteristics_original,
        "transfer_characteristics_Original"
    );
    mediainfo_attr!(
        transfer_characteristics_original_source,
        "transfer_characteristics_Original_Source"
    );

    mediainfo_attr!(matrix_coefficients, "matrix_coefficients");
    mediainfo_attr!(matrix_coefficients_source, "matrix_coefficients_Source");
    mediainfo_attr!(matrix_coefficients_original, "matrix_coefficients_Original");
    mediainfo_attr!(
        matrix_coefficients_original_source,
        "matrix_coefficients_Original_Source"
    );

    // Mastering display
    mediainfo_attr!(
        mastering_display_color_primaries,
        "MasteringDisplay_ColorPrimaries"
    );
    mediainfo_attr!(
        mastering_display_color_primaries_source,
        "MasteringDisplay_ColorPrimaries_Source"
    );
    mediainfo_attr!(
        mastering_display_color_primaries_original,
        "MasteringDisplay_ColorPrimaries_Original"
    );
    mediainfo_attr!(
        mastering_display_color_primaries_original_source,
        "MasteringDisplay_ColorPrimaries_Original_Source"
    );

    mediainfo_attr!(mastering_display_luminance, "MasteringDisplay_Luminance");
    mediainfo_attr!(
        mastering_display_luminance_source,
        "MasteringDisplay_Luminance_Source"
    );
    mediainfo_attr!(
        mastering_display_luminance_original,
        "MasteringDisplay_Luminance_Original"
    );
    mediainfo_attr!(
        mastering_display_luminance_original_source,
        "MasteringDisplay_Luminance_Original_Source"
    );

    // Content light levels
    mediainfo_attr!(max_cll, "MaxCLL");
    mediainfo_attr!(max_cll_source, "MaxCLL_Source");
    mediainfo_attr!(max_cll_original, "MaxCLL_Original");
    mediainfo_attr!(max_cll_original_source, "MaxCLL_Original_Source");

    mediainfo_attr!(max_fall, "MaxFALL");
    mediainfo_attr!(max_fall_source, "MaxFALL_Source");
    mediainfo_attr!(max_fall_original, "MaxFALL_Original");
    mediainfo_attr!(max_fall_original_source, "MaxFALL_Original_Source");

    // Resolution and format
    mediainfo_attr!(resolution, "Resolution");
    mediainfo_attr!(format, "Format");

    pub fn frame_size(&self) -> MediaInfoResult<String> {
        let height = self.height()?;
        let width = self.width()?;

        Ok(format!("{}x{}", width, height))
    }
}

/* TextStream */
impl TextStream {
    // Basic stream identification
    mediainfo_attr!(stream_id, "ID");

    // Format
    mediainfo_attr!(format, "Format");
    mediainfo_attr!(format_settings_wrapping, "Format_Settings_Wrapping");

    // Codec information
    mediainfo_attr!(codec_id, "CodecID");
    mediainfo_attr!(codec_info, "CodecID/Info");

    // Internet media type
    mediainfo_attr!(internet_media_type, "InternetMediaType");

    // Muxing mode
    mediainfo_attr!(muxing_mode, "MuxingMode");
    mediainfo_attr!(muxing_mode_more_info, "MuxingMode_MoreInfo");

    // Duration fields
    mediainfo_duration!(duration, "Duration");
    mediainfo_attr!(duration_string, "Duration/String");
    mediainfo_attr!(duration_string1, "Duration/String1");
    mediainfo_attr!(duration_string2, "Duration/String2");
    mediainfo_attr!(duration_string3, "Duration/String3");
    mediainfo_attr!(duration_string4, "Duration/String4");
    mediainfo_attr!(duration_string5, "Duration/String5");

    mediainfo_i64!(duration_start2_end, "Duration_Start2End");
    mediainfo_attr!(duration_start2_end_string, "Duration_Start2End/String");
    mediainfo_attr!(duration_start2_end_string1, "Duration_Start2End/String1");
    mediainfo_attr!(duration_start2_end_string2, "Duration_Start2End/String2");
    mediainfo_attr!(duration_start2_end_string3, "Duration_Start2End/String3");
    mediainfo_attr!(duration_start2_end_string4, "Duration_Start2End/String4");
    mediainfo_attr!(duration_start2_end_string5, "Duration_Start2End/String5");

    mediainfo_i64!(duration_start_command, "Duration_Start_Command");
    mediainfo_attr!(
        duration_start_command_string,
        "Duration_Start_Command/String"
    );
    mediainfo_attr!(
        duration_start_command_string1,
        "Duration_Start_Command/String1"
    );
    mediainfo_attr!(
        duration_start_command_string2,
        "Duration_Start_Command/String2"
    );
    mediainfo_attr!(
        duration_start_command_string3,
        "Duration_Start_Command/String3"
    );
    mediainfo_attr!(
        duration_start_command_string4,
        "Duration_Start_Command/String4"
    );
    mediainfo_attr!(
        duration_start_command_string5,
        "Duration_Start_Command/String5"
    );

    mediainfo_i64!(duration_start, "Duration_Start");
    mediainfo_attr!(duration_start_string, "Duration_Start/String");
    mediainfo_attr!(duration_start_string1, "Duration_Start/String1");
    mediainfo_attr!(duration_start_string2, "Duration_Start/String2");
    mediainfo_attr!(duration_start_string3, "Duration_Start/String3");
    mediainfo_attr!(duration_start_string4, "Duration_Start/String4");
    mediainfo_attr!(duration_start_string5, "Duration_Start/String5");

    mediainfo_i64!(duration_end, "Duration_End");
    mediainfo_attr!(duration_end_string, "Duration_End/String");
    mediainfo_attr!(duration_end_string1, "Duration_End/String1");
    mediainfo_attr!(duration_end_string2, "Duration_End/String2");
    mediainfo_attr!(duration_end_string3, "Duration_End/String3");
    mediainfo_attr!(duration_end_string4, "Duration_End/String4");
    mediainfo_attr!(duration_end_string5, "Duration_End/String5");

    mediainfo_i64!(duration_end_command, "Duration_End_Command");
    mediainfo_attr!(duration_end_command_string, "Duration_End_Command/String");
    mediainfo_attr!(duration_end_command_string1, "Duration_End_Command/String1");
    mediainfo_attr!(duration_end_command_string2, "Duration_End_Command/String2");
    mediainfo_attr!(duration_end_command_string3, "Duration_End_Command/String3");
    mediainfo_attr!(duration_end_command_string4, "Duration_End_Command/String4");
    mediainfo_attr!(duration_end_command_string5, "Duration_End_Command/String5");

    mediainfo_i64!(duration_first_frame, "Duration_FirstFrame");
    mediainfo_attr!(duration_first_frame_string, "Duration_FirstFrame/String");
    mediainfo_attr!(duration_first_frame_string1, "Duration_FirstFrame/String1");
    mediainfo_attr!(duration_first_frame_string2, "Duration_FirstFrame/String2");
    mediainfo_attr!(duration_first_frame_string3, "Duration_FirstFrame/String3");
    mediainfo_attr!(duration_first_frame_string4, "Duration_FirstFrame/String4");
    mediainfo_attr!(duration_first_frame_string5, "Duration_FirstFrame/String5");

    mediainfo_i64!(duration_last_frame, "Duration_LastFrame");
    mediainfo_attr!(duration_last_frame_string, "Duration_LastFrame/String");
    mediainfo_attr!(duration_last_frame_string1, "Duration_LastFrame/String1");
    mediainfo_attr!(duration_last_frame_string2, "Duration_LastFrame/String2");
    mediainfo_attr!(duration_last_frame_string3, "Duration_LastFrame/String3");
    mediainfo_attr!(duration_last_frame_string4, "Duration_LastFrame/String4");
    mediainfo_attr!(duration_last_frame_string5, "Duration_LastFrame/String5");

    // Duration base
    mediainfo_attr!(duration_base, "Duration_Base");

    // Source duration fields
    mediainfo_i64!(source_duration, "Source_Duration");
    mediainfo_attr!(source_duration_string, "Source_Duration/String");
    mediainfo_attr!(source_duration_string1, "Source_Duration/String1");
    mediainfo_attr!(source_duration_string2, "Source_Duration/String2");
    mediainfo_attr!(source_duration_string3, "Source_Duration/String3");
    mediainfo_attr!(source_duration_string4, "Source_Duration/String4");
    mediainfo_attr!(source_duration_string5, "Source_Duration/String5");

    mediainfo_i64!(source_duration_first_frame, "Source_Duration_FirstFrame");
    mediainfo_attr!(
        source_duration_first_frame_string,
        "Source_Duration_FirstFrame/String"
    );
    mediainfo_attr!(
        source_duration_first_frame_string1,
        "Source_Duration_FirstFrame/String1"
    );
    mediainfo_attr!(
        source_duration_first_frame_string2,
        "Source_Duration_FirstFrame/String2"
    );
    mediainfo_attr!(
        source_duration_first_frame_string3,
        "Source_Duration_FirstFrame/String3"
    );
    mediainfo_attr!(
        source_duration_first_frame_string4,
        "Source_Duration_FirstFrame/String4"
    );
    mediainfo_attr!(
        source_duration_first_frame_string5,
        "Source_Duration_FirstFrame/String5"
    );

    mediainfo_i64!(source_duration_last_frame, "Source_Duration_LastFrame");
    mediainfo_attr!(
        source_duration_last_frame_string,
        "Source_Duration_LastFrame/String"
    );
    mediainfo_attr!(
        source_duration_last_frame_string1,
        "Source_Duration_LastFrame/String1"
    );
    mediainfo_attr!(
        source_duration_last_frame_string2,
        "Source_Duration_LastFrame/String2"
    );
    mediainfo_attr!(
        source_duration_last_frame_string3,
        "Source_Duration_LastFrame/String3"
    );
    mediainfo_attr!(
        source_duration_last_frame_string4,
        "Source_Duration_LastFrame/String4"
    );
    mediainfo_attr!(
        source_duration_last_frame_string5,
        "Source_Duration_LastFrame/String5"
    );

    // Bit rate
    mediainfo_attr!(bit_rate_mode, "BitRate_Mode");
    mediainfo_attr!(bit_rate_mode_string, "BitRate_Mode/String");
    mediainfo_attr!(bit_rate, "BitRate");
    mediainfo_attr!(bit_rate_string, "BitRate/String");
    mediainfo_i64!(bit_rate_minimum, "BitRate_Minimum");
    mediainfo_attr!(bit_rate_minimum_string, "BitRate_Minimum/String");
    mediainfo_i64!(bit_rate_nominal, "BitRate_Nominal");
    mediainfo_attr!(bit_rate_nominal_string, "BitRate_Nominal/String");
    mediainfo_i64!(bit_rate_maximum, "BitRate_Maximum");
    mediainfo_attr!(bit_rate_maximum_string, "BitRate_Maximum/String");
    mediainfo_i64!(bit_rate_encoded, "BitRate_Encoded");
    mediainfo_attr!(bit_rate_encoded_string, "BitRate_Encoded/String");

    // Dimensions
    mediainfo_i64!(width, "Width");
    mediainfo_attr!(width_string, "Width/String");
    mediainfo_i64!(height, "Height");
    mediainfo_attr!(height_string, "Height/String");

    // Display aspect ratio
    mediainfo_attr!(display_aspect_ratio, "DisplayAspectRatio");
    mediainfo_attr!(display_aspect_ratio_string, "DisplayAspectRatio/String");
    mediainfo_attr!(display_aspect_ratio_original, "DisplayAspectRatio_Original");
    mediainfo_attr!(
        display_aspect_ratio_original_string,
        "DisplayAspectRatio_Original/String"
    );

    // Frame rate
    mediainfo_attr!(frame_rate_mode, "FrameRate_Mode");
    mediainfo_attr!(frame_rate_mode_string, "FrameRate_Mode/String");
    mediainfo_attr!(frame_rate_mode_original, "FrameRate_Mode_Original");
    mediainfo_attr!(
        frame_rate_mode_original_string,
        "FrameRate_Mode_Original/String"
    );

    mediainfo_attr!(frame_rate, "FrameRate");
    mediainfo_attr!(frame_rate_string, "FrameRate/String");
    mediainfo_i64!(frame_rate_num, "FrameRate_Num");
    mediainfo_i64!(frame_rate_den, "FrameRate_Den");
    mediainfo_attr!(frame_rate_minimum, "FrameRate_Minimum");
    mediainfo_attr!(frame_rate_minimum_string, "FrameRate_Minimum/String");
    mediainfo_attr!(frame_rate_nominal, "FrameRate_Nominal");
    mediainfo_attr!(frame_rate_nominal_string, "FrameRate_Nominal/String");
    mediainfo_attr!(frame_rate_maximum, "FrameRate_Maximum");
    mediainfo_attr!(frame_rate_maximum_string, "FrameRate_Maximum/String");

    mediainfo_attr!(frame_rate_original, "FrameRate_Original");
    mediainfo_attr!(frame_rate_original_string, "FrameRate_Original/String");
    mediainfo_i64!(frame_rate_original_num, "FrameRate_Original_Num");
    mediainfo_i64!(frame_rate_original_den, "FrameRate_Original_Den");

    mediainfo_i64!(frame_count, "FrameCount");
    mediainfo_i64!(element_count, "ElementCount");
    mediainfo_i64!(source_frame_count, "Source_FrameCount");

    // Color
    mediainfo_attr!(color_space, "ColorSpace");
    mediainfo_attr!(chroma_subsampling, "ChromaSubsampling");
    mediainfo_i64!(bit_depth, "BitDepth");
    mediainfo_attr!(bit_depth_string, "BitDepth/String");

    // Compression
    mediainfo_attr!(compression_mode, "Compression_Mode");
    mediainfo_attr!(compression_mode_string, "Compression_Mode/String");
    mediainfo_attr!(compression_ratio, "Compression_Ratio");

    // Title
    mediainfo_attr!(title, "Title");

    // Technical
    mediainfo_attr!(encoded_application, "Encoded_Application");
    mediainfo_attr!(encoded_application_string, "Encoded_Application/String");
    mediainfo_attr!(
        encoded_application_company_name,
        "Encoded_Application_CompanyName"
    );
    mediainfo_attr!(encoded_application_name, "Encoded_Application_Name");
    mediainfo_attr!(encoded_application_version, "Encoded_Application_Version");
    mediainfo_attr!(encoded_application_url, "Encoded_Application_Url");

    mediainfo_attr!(encoded_library, "Encoded_Library");
    mediainfo_attr!(encoded_library_string, "Encoded_Library/String");
    mediainfo_attr!(encoded_library_company_name, "Encoded_Library_CompanyName");
    mediainfo_attr!(encoded_library_name, "Encoded_Library_Name");
    mediainfo_attr!(encoded_library_version, "Encoded_Library_Version");
    mediainfo_attr!(encoded_library_date, "Encoded_Library_Date");
    mediainfo_attr!(encoded_library_settings, "Encoded_Library_Settings");
    mediainfo_attr!(encoded_operating_system, "Encoded_OperatingSystem");

    // Language
    mediainfo_attr!(language, "Language");
    mediainfo_attr!(language_string, "Language/String");
    mediainfo_attr!(language_string1, "Language/String1");
    mediainfo_attr!(language_string2, "Language/String2");
    mediainfo_attr!(language_string3, "Language/String3");
    mediainfo_attr!(language_string4, "Language/String4");
    mediainfo_attr!(language_more, "Language_More");

    // Service kind
    mediainfo_attr!(service_kind, "ServiceKind");
    mediainfo_attr!(service_kind_string, "ServiceKind/String");

    // Flags
    mediainfo_attr!(disabled, "Disabled");
    mediainfo_attr!(disabled_string, "Disabled/String");
    mediainfo_attr!(default, "Default");
    mediainfo_attr!(default_string, "Default/String");
    mediainfo_attr!(forced, "Forced");
    mediainfo_attr!(forced_string, "Forced/String");
    mediainfo_attr!(alternate_group, "AlternateGroup");
    mediainfo_attr!(alternate_group_string, "AlternateGroup/String");

    // Summary
    mediainfo_attr!(summary, "Summary");

    // Dates
    mediainfo_date!(encoded_date, "Encoded_Date");
    mediainfo_date!(tagged_date, "Tagged_Date");

    // Encryption
    mediainfo_attr!(encryption, "Encryption");

    // Events
    mediainfo_attr!(events_total, "Events_Total");
    mediainfo_i64!(events_min_duration, "Events_MinDuration");
    mediainfo_attr!(events_min_duration_string, "Events_MinDuration/String");
    mediainfo_attr!(events_min_duration_string1, "Events_MinDuration/String1");
    mediainfo_attr!(events_min_duration_string2, "Events_MinDuration/String2");
    mediainfo_attr!(events_min_duration_string3, "Events_MinDuration/String3");
    mediainfo_attr!(events_min_duration_string4, "Events_MinDuration/String4");
    mediainfo_attr!(events_min_duration_string5, "Events_MinDuration/String5");

    mediainfo_attr!(events_pop_on, "Events_PopOn");
    mediainfo_attr!(events_roll_up, "Events_RollUp");
    mediainfo_attr!(events_paint_on, "Events_PaintOn");

    // Lines
    mediainfo_attr!(lines_count, "Lines_Count");
    mediainfo_i64!(lines_max_count_per_event, "Lines_MaxCountPerEvent");
    mediainfo_i64!(lines_max_character_count, "Lines_MaxCharacterCount");

    // First display
    mediainfo_attr!(first_display_delay_frames, "FirstDisplay_Delay_Frames");
    mediainfo_attr!(first_display_type, "FirstDisplay_Type");
}

/* OtherStream */
impl OtherStream {
    // Basic stream identification
    mediainfo_attr!(stream_id, "ID");
    mediainfo_attr!(other_type, "Type");

    // Format settings
    mediainfo_attr!(format_settings_wrapping, "Format_Settings_Wrapping");

    // Muxing mode
    mediainfo_attr!(muxing_mode, "MuxingMode");
    mediainfo_attr!(muxing_mode_more_info, "MuxingMode_MoreInfo");

    // Duration fields
    mediainfo_duration!(duration, "Duration");
    mediainfo_attr!(duration_string, "Duration/String");
    mediainfo_attr!(duration_string1, "Duration/String1");
    mediainfo_attr!(duration_string2, "Duration/String2");
    mediainfo_attr!(duration_string3, "Duration/String3");
    mediainfo_attr!(duration_string4, "Duration/String4");
    mediainfo_attr!(duration_string5, "Duration/String5");

    mediainfo_i64!(duration_start, "Duration_Start");
    mediainfo_i64!(duration_end, "Duration_End");

    // Source duration
    mediainfo_i64!(source_duration, "Source_Duration");
    mediainfo_attr!(source_duration_string, "Source_Duration/String");
    mediainfo_attr!(source_duration_string1, "Source_Duration/String1");
    mediainfo_attr!(source_duration_string2, "Source_Duration/String2");
    mediainfo_attr!(source_duration_string3, "Source_Duration/String3");
    mediainfo_attr!(source_duration_string4, "Source_Duration/String4");
    mediainfo_attr!(source_duration_string5, "Source_Duration/String5");

    mediainfo_i64!(source_duration_first_frame, "Source_Duration_FirstFrame");
    mediainfo_attr!(
        source_duration_first_frame_string,
        "Source_Duration_FirstFrame/String"
    );
    mediainfo_attr!(
        source_duration_first_frame_string1,
        "Source_Duration_FirstFrame/String1"
    );
    mediainfo_attr!(
        source_duration_first_frame_string2,
        "Source_Duration_FirstFrame/String2"
    );
    mediainfo_attr!(
        source_duration_first_frame_string3,
        "Source_Duration_FirstFrame/String3"
    );
    mediainfo_attr!(
        source_duration_first_frame_string4,
        "Source_Duration_FirstFrame/String4"
    );
    mediainfo_attr!(
        source_duration_first_frame_string5,
        "Source_Duration_FirstFrame/String5"
    );

    mediainfo_i64!(source_duration_last_frame, "Source_Duration_LastFrame");
    mediainfo_attr!(
        source_duration_last_frame_string,
        "Source_Duration_LastFrame/String"
    );
    mediainfo_attr!(
        source_duration_last_frame_string1,
        "Source_Duration_LastFrame/String1"
    );
    mediainfo_attr!(
        source_duration_last_frame_string2,
        "Source_Duration_LastFrame/String2"
    );
    mediainfo_attr!(
        source_duration_last_frame_string3,
        "Source_Duration_LastFrame/String3"
    );
    mediainfo_attr!(
        source_duration_last_frame_string4,
        "Source_Duration_LastFrame/String4"
    );
    mediainfo_attr!(
        source_duration_last_frame_string5,
        "Source_Duration_LastFrame/String5"
    );

    // Bit rate
    mediainfo_attr!(bit_rate_mode, "BitRate_Mode");
    mediainfo_attr!(bit_rate_mode_string, "BitRate_Mode/String");
    mediainfo_attr!(bit_rate, "BitRate");
    mediainfo_attr!(bit_rate_string, "BitRate/String");
    mediainfo_i64!(bit_rate_minimum, "BitRate_Minimum");
    mediainfo_attr!(bit_rate_minimum_string, "BitRate_Minimum/String");
    mediainfo_i64!(bit_rate_nominal, "BitRate_Nominal");
    mediainfo_attr!(bit_rate_nominal_string, "BitRate_Nominal/String");
    mediainfo_i64!(bit_rate_maximum, "BitRate_Maximum");
    mediainfo_attr!(bit_rate_maximum_string, "BitRate_Maximum/String");
    mediainfo_i64!(bit_rate_encoded, "BitRate_Encoded");
    mediainfo_attr!(bit_rate_encoded_string, "BitRate_Encoded/String");

    // Frame rate
    mediainfo_attr!(frame_rate, "FrameRate");
    mediainfo_attr!(frame_rate_string, "FrameRate/String");
    mediainfo_i64!(frame_rate_num, "FrameRate_Num");
    mediainfo_i64!(frame_rate_den, "FrameRate_Den");
    mediainfo_i64!(frame_count, "FrameCount");
    mediainfo_i64!(source_frame_count, "Source_FrameCount");

    // Time code
    mediainfo_attr!(timecode, "TimeCode_FirstFrame");
    mediainfo_attr!(time_code_last_frame, "TimeCode_LastFrame");
    mediainfo_attr!(time_code_drop_frame, "TimeCode_DropFrame");
    mediainfo_attr!(time_code_settings, "TimeCode_Settings");
    mediainfo_attr!(time_code_stripped, "TimeCode_Stripped");
    mediainfo_attr!(time_code_stripped_string, "TimeCode_Stripped/String");
    mediainfo_attr!(time_code_source, "TimeCode_Source");

    // Stream size
    mediainfo_i64!(stream_size, "StreamSize");
    mediainfo_attr!(stream_size_string, "StreamSize/String");
    mediainfo_attr!(stream_size_string1, "StreamSize/String1");
    mediainfo_attr!(stream_size_string2, "StreamSize/String2");
    mediainfo_attr!(stream_size_string3, "StreamSize/String3");
    mediainfo_attr!(stream_size_string4, "StreamSize/String4");
    mediainfo_attr!(stream_size_string5, "StreamSize/String5");
    mediainfo_attr!(stream_size_proportion, "StreamSize_Proportion");

    mediainfo_i64!(stream_size_demuxed, "StreamSize_Demuxed");
    mediainfo_attr!(stream_size_demuxed_string, "StreamSize_Demuxed/String");
    mediainfo_attr!(stream_size_demuxed_string1, "StreamSize_Demuxed/String1");
    mediainfo_attr!(stream_size_demuxed_string2, "StreamSize_Demuxed/String2");
    mediainfo_attr!(stream_size_demuxed_string3, "StreamSize_Demuxed/String3");
    mediainfo_attr!(stream_size_demuxed_string4, "StreamSize_Demuxed/String4");
    mediainfo_attr!(stream_size_demuxed_string5, "StreamSize_Demuxed/String5");

    mediainfo_i64!(source_stream_size, "Source_StreamSize");
    mediainfo_attr!(source_stream_size_string, "Source_StreamSize/String");
    mediainfo_attr!(source_stream_size_string1, "Source_StreamSize/String1");
    mediainfo_attr!(source_stream_size_string2, "Source_StreamSize/String2");
    mediainfo_attr!(source_stream_size_string3, "Source_StreamSize/String3");
    mediainfo_attr!(source_stream_size_string4, "Source_StreamSize/String4");
    mediainfo_attr!(source_stream_size_string5, "Source_StreamSize/String5");
    mediainfo_attr!(
        source_stream_size_proportion,
        "Source_StreamSize_Proportion"
    );

    mediainfo_i64!(stream_size_encoded, "StreamSize_Encoded");
    mediainfo_attr!(stream_size_encoded_string, "StreamSize_Encoded/String");
    mediainfo_attr!(stream_size_encoded_string1, "StreamSize_Encoded/String1");
    mediainfo_attr!(stream_size_encoded_string2, "StreamSize_Encoded/String2");
    mediainfo_attr!(stream_size_encoded_string3, "StreamSize_Encoded/String3");
    mediainfo_attr!(stream_size_encoded_string4, "StreamSize_Encoded/String4");
    mediainfo_attr!(stream_size_encoded_string5, "StreamSize_Encoded/String5");
    mediainfo_attr!(
        stream_size_encoded_proportion,
        "StreamSize_Encoded_Proportion"
    );

    mediainfo_i64!(source_stream_size_encoded, "Source_StreamSize_Encoded");
    mediainfo_attr!(
        source_stream_size_encoded_string,
        "Source_StreamSize_Encoded/String"
    );
    mediainfo_attr!(
        source_stream_size_encoded_string1,
        "Source_StreamSize_Encoded/String1"
    );
    mediainfo_attr!(
        source_stream_size_encoded_string2,
        "Source_StreamSize_Encoded/String2"
    );
    mediainfo_attr!(
        source_stream_size_encoded_string3,
        "Source_StreamSize_Encoded/String3"
    );
    mediainfo_attr!(
        source_stream_size_encoded_string4,
        "Source_StreamSize_Encoded/String4"
    );
    mediainfo_attr!(
        source_stream_size_encoded_string5,
        "Source_StreamSize_Encoded/String5"
    );
    mediainfo_attr!(
        source_stream_size_encoded_proportion,
        "Source_StreamSize_Encoded_Proportion"
    );

    // Title
    mediainfo_attr!(title, "Title");

    // Language
    mediainfo_attr!(language, "Language");
    mediainfo_attr!(language_string, "Language/String");
    mediainfo_attr!(language_string1, "Language/String1");
    mediainfo_attr!(language_string2, "Language/String2");
    mediainfo_attr!(language_string3, "Language/String3");
    mediainfo_attr!(language_string4, "Language/String4");
    mediainfo_attr!(language_more, "Language_More");

    // Service kind
    mediainfo_attr!(service_kind, "ServiceKind");
    mediainfo_attr!(service_kind_string, "ServiceKind/String");

    // Flags
    mediainfo_attr!(disabled, "Disabled");
    mediainfo_attr!(disabled_string, "Disabled/String");
    mediainfo_attr!(default, "Default");
    mediainfo_attr!(default_string, "Default/String");
    mediainfo_attr!(forced, "Forced");
    mediainfo_attr!(forced_string, "Forced/String");
    mediainfo_attr!(alternate_group, "AlternateGroup");
    mediainfo_attr!(alternate_group_string, "AlternateGroup/String");
}

/* MenuStream */
impl MenuStream {
    // Basic stream identification
    mediainfo_attr!(stream_id, "ID");

    // Duration fields
    mediainfo_duration!(duration, "Duration");
    mediainfo_attr!(duration_string, "Duration/String");
    mediainfo_attr!(duration_string1, "Duration/String1");
    mediainfo_attr!(duration_string2, "Duration/String2");
    mediainfo_attr!(duration_string3, "Duration/String3");
    mediainfo_attr!(duration_string4, "Duration/String4");
    mediainfo_attr!(duration_string5, "Duration/String5");

    mediainfo_i64!(duration_start, "Duration_Start");
    mediainfo_i64!(duration_end, "Duration_End");

    // Delay
    mediainfo_i64!(delay, "Delay");
    mediainfo_attr!(delay_string, "Delay/String");
    mediainfo_attr!(delay_string1, "Delay/String1");
    mediainfo_attr!(delay_string2, "Delay/String2");
    mediainfo_attr!(delay_string3, "Delay/String3");
    mediainfo_attr!(delay_string4, "Delay/String4");
    mediainfo_attr!(delay_string5, "Delay/String5");
    mediainfo_attr!(delay_settings, "Delay_Settings");
    mediainfo_attr!(delay_drop_frame, "Delay_DropFrame");
    mediainfo_attr!(delay_source, "Delay_Source");

    // Frame rate
    mediainfo_attr!(frame_rate_mode, "FrameRate_Mode");
    mediainfo_attr!(frame_rate_mode_string, "FrameRate_Mode/String");
    mediainfo_attr!(frame_rate, "FrameRate");
    mediainfo_attr!(frame_rate_string, "FrameRate/String");
    mediainfo_i64!(frame_rate_num, "FrameRate_Num");
    mediainfo_i64!(frame_rate_den, "FrameRate_Den");
    mediainfo_i64!(frame_count, "FrameCount");

    // Lists
    mediainfo_attr!(list_stream_kind, "List_StreamKind");
    mediainfo_attr!(list_stream_pos, "List_StreamPos");
    mediainfo_attr!(list, "List");
    mediainfo_attr!(list_string, "List/String");

    // Title
    mediainfo_attr!(title, "Title");

    // Language
    mediainfo_attr!(language, "Language");
    mediainfo_attr!(language_string, "Language/String");
    mediainfo_attr!(language_string1, "Language/String1");
    mediainfo_attr!(language_string2, "Language/String2");
    mediainfo_attr!(language_string3, "Language/String3");
    mediainfo_attr!(language_string4, "Language/String4");
    mediainfo_attr!(language_more, "Language_More");

    // Service
    mediainfo_attr!(service_kind, "ServiceKind");
    mediainfo_attr!(service_kind_string, "ServiceKind/String");
    mediainfo_attr!(service_name, "ServiceName");
    mediainfo_attr!(service_channel, "ServiceChannel");
    mediainfo_attr!(service_url, "Service_Url");
    mediainfo_attr!(service_provider, "ServiceProvider");
    mediainfo_attr!(service_provider_url, "ServiceProvider_Url");
    mediainfo_attr!(service_type, "ServiceType");

    // Networks
    mediainfo_attr!(network_name, "NetworkName");
    mediainfo_attr!(original_network_name, "Original_NetworkName");

    // Location
    mediainfo_attr!(countries, "Countries");
    mediainfo_attr!(time_zones, "TimeZones");

    // Rating
    mediainfo_attr!(law_rating, "LawRating");
    mediainfo_attr!(law_rating_reason, "LawRating_Reason");

    // Flags
    mediainfo_attr!(disabled, "Disabled");
    mediainfo_attr!(disabled_string, "Disabled/String");
    mediainfo_attr!(default, "Default");
    mediainfo_attr!(default_string, "Default/String");
    mediainfo_attr!(forced, "Forced");
    mediainfo_attr!(forced_string, "Forced/String");
    mediainfo_attr!(alternate_group, "AlternateGroup");
    mediainfo_attr!(alternate_group_string, "AlternateGroup/String");

    // Chapters
    mediainfo_i64!(chapters_pos_begin, "Chapters_Pos_Begin");
    mediainfo_i64!(chapters_pos_end, "Chapters_Pos_End");

    // Dates
    mediainfo_date!(encoded_date, "Encoded_Date");
    mediainfo_date!(tagged_date, "Tagged_Date");
}
