use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    
    if target.contains("wasm32") {
        build_for_wasm();
    } else {
        build_for_native();
    }
}

fn build_for_native() {
    let lib_mediainfo = pkg_config::probe_library("libmediainfo");
    if lib_mediainfo.is_err() {
        panic!("Could not find MediaInfo via pkgconfig");
    }
}

fn build_for_wasm() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mediainfo_src = PathBuf::from(&manifest_dir).join("mediainfo_src");
    
    println!("cargo:rerun-if-changed={}", mediainfo_src.display());
    
    // Compile ZenLib first (dependency of MediaInfo)
    let mut zen_build = cc::Build::new();
    zen_build
        .cpp(true)
        .include(mediainfo_src.join("ZenLib/Source"))
        .include(mediainfo_src.join("ZenLib/Source/ZenLib"))
        .define("UNICODE", None)
        .define("_UNICODE", None)
        .define("ZENLIB_STATIC", None)
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-w"); // Suppress warnings
    
    // Add ALL ZenLib source files
    let zenlib_sources = [
        "ZenLib/Source/ZenLib/Conf.cpp",
        "ZenLib/Source/ZenLib/CriticalSection.cpp",
        "ZenLib/Source/ZenLib/Dir.cpp",
        "ZenLib/Source/ZenLib/File.cpp",
        "ZenLib/Source/ZenLib/FileName.cpp",
        "ZenLib/Source/ZenLib/Format/Html/Html_Handler.cpp",
        "ZenLib/Source/ZenLib/Format/Html/Html_Request.cpp",
        "ZenLib/Source/ZenLib/Format/Http/Http_Cookies.cpp",
        "ZenLib/Source/ZenLib/Format/Http/Http_Handler.cpp",
        "ZenLib/Source/ZenLib/Format/Http/Http_Request.cpp",
        "ZenLib/Source/ZenLib/Format/Http/Http_Utils.cpp",
        "ZenLib/Source/ZenLib/HTTP_Client.cpp",
        "ZenLib/Source/ZenLib/InfoMap.cpp",
        "ZenLib/Source/ZenLib/int128s.cpp",
        "ZenLib/Source/ZenLib/int128u.cpp",
        "ZenLib/Source/ZenLib/MemoryDebug.cpp",
        "ZenLib/Source/ZenLib/OS_Utils.cpp",
        "ZenLib/Source/ZenLib/PreComp.cpp",
        "ZenLib/Source/ZenLib/Thread.cpp",
        "ZenLib/Source/ZenLib/Translation.cpp",
        "ZenLib/Source/ZenLib/Utils.cpp",
        "ZenLib/Source/ZenLib/Ztring.cpp",
        "ZenLib/Source/ZenLib/ZtringList.cpp",
        "ZenLib/Source/ZenLib/ZtringListList.cpp",
        "ZenLib/Source/ZenLib/ZtringListListF.cpp",
    ];
    
    for source in &zenlib_sources {
        let source_path = mediainfo_src.join(source);
        if source_path.exists() {
            zen_build.file(&source_path);
        }
    }
    
    zen_build.compile("zen");
    
    // Compile MediaInfo
    let mut mediainfo_build = cc::Build::new();
    mediainfo_build
        .cpp(true)
        .include(mediainfo_src.join("MediaInfoLib/Source"))
        .include(mediainfo_src.join("MediaInfoLib/Source/MediaInfo"))
        .include(mediainfo_src.join("ZenLib/Source"))
        .define("UNICODE", None)
        .define("_UNICODE", None)
        .define("MEDIAINFO_STATIC", None)
        .define("MEDIAINFODLL_EXPORTS", None)
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-w"); // Suppress warnings
    
    // Add ALL MediaInfo source files (complete list of 250 files)
    let mediainfo_sources = [
        "MediaInfoLib/Source/MediaInfo/Archive/File_7z.cpp",
        "MediaInfoLib/Source/MediaInfo/Archive/File_Ace.cpp",
        "MediaInfoLib/Source/MediaInfo/Archive/File_Bzip2.cpp",
        "MediaInfoLib/Source/MediaInfo/Archive/File_Elf.cpp",
        "MediaInfoLib/Source/MediaInfo/Archive/File_Gzip.cpp",
        "MediaInfoLib/Source/MediaInfo/Archive/File_Iso9660.cpp",
        "MediaInfoLib/Source/MediaInfo/Archive/File_MachO.cpp",
        "MediaInfoLib/Source/MediaInfo/Archive/File_Mz.cpp",
        "MediaInfoLib/Source/MediaInfo/Archive/File_Rar.cpp",
        "MediaInfoLib/Source/MediaInfo/Archive/File_Tar.cpp",
        "MediaInfoLib/Source/MediaInfo/Archive/File_Zip.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Aac_GeneralAudio_Sbr_Ps.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Aac_GeneralAudio_Sbr.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Aac_GeneralAudio.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Aac_Main.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Aac_Others.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Aac.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Ac3.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Ac4.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Adm.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Adpcm.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Als.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Amr.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Amv.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Ape.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Aptx100.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Au.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Caf.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Celt.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_ChannelGrouping.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_ChannelSplitting.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Dat.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_DolbyAudioMetadata.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_DolbyE.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Dsdiff.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Dsf.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Dts.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_DtsUhd.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_ExtendedModule.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Flac.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Iab.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Iamf.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_ImpulseTracker.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_La.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Mga.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Midi.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Module.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Mpc.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_MpcSv8.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Mpega.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Mpegh3da.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_OpenMG.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Opus.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Pcm_M2ts.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Pcm_Vob.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Pcm.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Ps2Audio.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Rkau.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_ScreamTracker3.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_SmpteSt0302.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_SmpteSt0331.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_SmpteSt0337.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Speex.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Tak.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Tta.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_TwinVQ.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Usac.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Vorbis.cpp",
        "MediaInfoLib/Source/MediaInfo/Audio/File_Wvpk.cpp",
        "MediaInfoLib/Source/MediaInfo/Duplicate/File__Duplicate__Base.cpp",
        "MediaInfoLib/Source/MediaInfo/Duplicate/File__Duplicate__Writer.cpp",
        "MediaInfoLib/Source/MediaInfo/Duplicate/File__Duplicate_MpegTs.cpp",
        "MediaInfoLib/Source/MediaInfo/Export/Export_EbuCore.cpp",
        "MediaInfoLib/Source/MediaInfo/Export/Export_Fims.cpp",
        "MediaInfoLib/Source/MediaInfo/Export/Export_Graph.cpp",
        "MediaInfoLib/Source/MediaInfo/Export/Export_Mpeg7.cpp",
        "MediaInfoLib/Source/MediaInfo/Export/Export_Niso.cpp",
        "MediaInfoLib/Source/MediaInfo/Export/Export_PBCore.cpp",
        "MediaInfoLib/Source/MediaInfo/Export/Export_PBCore2.cpp",
        "MediaInfoLib/Source/MediaInfo/Export/Export_reVTMD.cpp",
        "MediaInfoLib/Source/MediaInfo/ExternalCommandHelpers.cpp",
        "MediaInfoLib/Source/MediaInfo/File__Analyze_Buffer_MinimizeSize.cpp",
        "MediaInfoLib/Source/MediaInfo/File__Analyze_Buffer.cpp",
        "MediaInfoLib/Source/MediaInfo/File__Analyze_Element.cpp",
        "MediaInfoLib/Source/MediaInfo/File__Analyze_Streams_Finish.cpp",
        "MediaInfoLib/Source/MediaInfo/File__Analyze_Streams.cpp",
        "MediaInfoLib/Source/MediaInfo/File__Analyze.cpp",
        "MediaInfoLib/Source/MediaInfo/File__Base.cpp",
        "MediaInfoLib/Source/MediaInfo/File__Duplicate.cpp",
        "MediaInfoLib/Source/MediaInfo/File__HasReferences.cpp",
        "MediaInfoLib/Source/MediaInfo/File__MultipleParsing.cpp",
        "MediaInfoLib/Source/MediaInfo/File_Dummy.cpp",
        "MediaInfoLib/Source/MediaInfo/File_Other.cpp",
        "MediaInfoLib/Source/MediaInfo/File_Unknown.cpp",
        "MediaInfoLib/Source/MediaInfo/HashWrapper.cpp",
        "MediaInfoLib/Source/MediaInfo/Image/File_ArriRaw.cpp",
        "MediaInfoLib/Source/MediaInfo/Image/File_Bmp.cpp",
        "MediaInfoLib/Source/MediaInfo/Image/File_Bpg.cpp",
        "MediaInfoLib/Source/MediaInfo/Image/File_Dds.cpp",
        "MediaInfoLib/Source/MediaInfo/Image/File_Dpx.cpp",
        "MediaInfoLib/Source/MediaInfo/Image/File_Exr.cpp",
        "MediaInfoLib/Source/MediaInfo/Image/File_GainMap.cpp",
        "MediaInfoLib/Source/MediaInfo/Image/File_Gif.cpp",
        "MediaInfoLib/Source/MediaInfo/Image/File_Ico.cpp",
        "MediaInfoLib/Source/MediaInfo/Image/File_Jpeg.cpp",
        "MediaInfoLib/Source/MediaInfo/Image/File_Pcx.cpp",
        "MediaInfoLib/Source/MediaInfo/Image/File_Png.cpp",
        "MediaInfoLib/Source/MediaInfo/Image/File_Psd.cpp",
        "MediaInfoLib/Source/MediaInfo/Image/File_Rle.cpp",
        "MediaInfoLib/Source/MediaInfo/Image/File_Tga.cpp",
        "MediaInfoLib/Source/MediaInfo/Image/File_Tiff.cpp",
        "MediaInfoLib/Source/MediaInfo/Image/File_WebP.cpp",
        "MediaInfoLib/Source/MediaInfo/MediaInfo_Config_Automatic.cpp",
        "MediaInfoLib/Source/MediaInfo/MediaInfo_Config_MediaInfo.cpp",
        "MediaInfoLib/Source/MediaInfo/MediaInfo_Config_PerPackage.cpp",
        "MediaInfoLib/Source/MediaInfo/MediaInfo_Config.cpp",
        "MediaInfoLib/Source/MediaInfo/MediaInfo_File.cpp",
        "MediaInfoLib/Source/MediaInfo/MediaInfo_Inform.cpp",
        "MediaInfoLib/Source/MediaInfo/MediaInfo_Internal.cpp",
        "MediaInfoLib/Source/MediaInfo/MediaInfo.cpp",
        "MediaInfoLib/Source/MediaInfo/MediaInfoList_Internal.cpp",
        "MediaInfoLib/Source/MediaInfo/MediaInfoList.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File__ReferenceFilesHelper_Resource.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File__ReferenceFilesHelper_Sequence.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File__ReferenceFilesHelper.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Aaf.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Ancillary.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Bdmv.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Cdxa.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_DashMpd.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_DcpAm.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_DcpCpl.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_DcpPkl.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Dpg.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_DvDif_Analysis.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_DvDif.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Dvdv.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Dxw.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Flv.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Gxf_TimeCode.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Gxf.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_HdsF4m.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Hls.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Ibi_Creation.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Ibi.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Ism.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Ivf.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Lxf.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_MiXml.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Mk.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Mpeg_Descriptors.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Mpeg_Psi.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Mpeg4_Descriptors.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Mpeg4_Elements.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Mpeg4_TimeCode.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Mpeg4.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_MpegPs.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_MpegTs_Duplicate.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_MpegTs.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Mxf.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Nsv.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Nut.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Ogg_SubElement.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Ogg.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_P2_Clip.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Pmp.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Ptx.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Riff_Elements.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Riff.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Rm.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_SequenceInfo.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Skm.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Swf.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Umf.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Vbi.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Wm_Elements.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Wm.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Wtv.cpp",
        "MediaInfoLib/Source/MediaInfo/Multiple/File_Xdcam_Clip.cpp",
        "MediaInfoLib/Source/MediaInfo/OutputHelpers.cpp",
        "MediaInfoLib/Source/MediaInfo/PreComp.cpp",
        "MediaInfoLib/Source/MediaInfo/Reader/Reader_Directory.cpp",
        "MediaInfoLib/Source/MediaInfo/Reader/Reader_File.cpp",
        "MediaInfoLib/Source/MediaInfo/Reader/Reader_libcurl.cpp",
        "MediaInfoLib/Source/MediaInfo/Reader/Reader_libmms.cpp",
        "MediaInfoLib/Source/MediaInfo/Tag/File__Tags.cpp",
        "MediaInfoLib/Source/MediaInfo/Tag/File_ApeTag.cpp",
        "MediaInfoLib/Source/MediaInfo/Tag/File_C2pa.cpp",
        "MediaInfoLib/Source/MediaInfo/Tag/File_Exif.cpp",
        "MediaInfoLib/Source/MediaInfo/Tag/File_Icc.cpp",
        "MediaInfoLib/Source/MediaInfo/Tag/File_Id3.cpp",
        "MediaInfoLib/Source/MediaInfo/Tag/File_Id3v2.cpp",
        "MediaInfoLib/Source/MediaInfo/Tag/File_Iim.cpp",
        "MediaInfoLib/Source/MediaInfo/Tag/File_Lyrics3.cpp",
        "MediaInfoLib/Source/MediaInfo/Tag/File_Lyrics3v2.cpp",
        "MediaInfoLib/Source/MediaInfo/Tag/File_PropertyList.cpp",
        "MediaInfoLib/Source/MediaInfo/Tag/File_SphericalVideo.cpp",
        "MediaInfoLib/Source/MediaInfo/Tag/File_VorbisCom.cpp",
        "MediaInfoLib/Source/MediaInfo/Tag/File_Xmp.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_AribStdB24B37.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_Cdp.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_Cmml.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_DtvccTransport.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_DvbSubtitle.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_Eia608.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_Eia708.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_Kate.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_N19.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_OtherText.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_Pac.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_Pdf.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_Pgs.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_Scc.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_Scte20.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_Sdp.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_SubRip.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_Teletext.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_TimedText.cpp",
        "MediaInfoLib/Source/MediaInfo/Text/File_Ttml.cpp",
        "MediaInfoLib/Source/MediaInfo/TimeCode.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_AfdBarData.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Aic.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Av1.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Avc_Duplicate.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Avc.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Avs3V.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_AvsV.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Canopus.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_CineForm.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Dirac.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_DolbyVisionMetadata.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Ffv1.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Flic.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Fraps.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_H263.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_HdrVividMetadata.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Hevc.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_HuffYuv.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Lagarith.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Mpeg4v.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Mpegv.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_ProRes.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Theora.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Vc1.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Vc3.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Vp8.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Vp9.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Vvc.cpp",
        "MediaInfoLib/Source/MediaInfo/Video/File_Y4m.cpp",
        "MediaInfoLib/Source/MediaInfo/XmlUtils.cpp",
    ];
    
    for source in &mediainfo_sources {
        let source_path = mediainfo_src.join(source);
        if source_path.exists() {
            mediainfo_build.file(&source_path);
        }
    }
    
    mediainfo_build.compile("mediainfo");
    
    // Add MediaInfoDLL wrapper
    let mut dll_build = cc::Build::new();
    dll_build
        .cpp(true)
        .include(mediainfo_src.join("MediaInfoLib/Source"))
        .include(mediainfo_src.join("ZenLib/Source"))
        .define("UNICODE", None)
        .define("_UNICODE", None)
        .define("MEDIAINFO_STATIC", None)
        .define("MEDIAINFODLL_EXPORTS", None)
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-w")
        .file(mediainfo_src.join("MediaInfoLib/Source/MediaInfoDLL/MediaInfoDLL.cpp"));
    
    dll_build.compile("mediainfodll");
    
    // Add WASM-specific linker flags
    println!("cargo:rustc-link-arg=-sALLOW_MEMORY_GROWTH=1");
    println!("cargo:rustc-link-arg=-sMALLOC=emmalloc");
    println!("cargo:rustc-link-arg=-sASSERTIONS=0");
    println!("cargo:rustc-link-arg=-sNO_FILESYSTEM=1");
    println!("cargo:rustc-link-arg=-sINITIAL_MEMORY=33554432");
    
    // Export symbols so they don't get imported from env
    println!("cargo:rustc-link-arg=-sEXPORTED_FUNCTIONS=_MediaInfo_New,_MediaInfo_Delete,_MediaInfo_Open_Buffer_Init,_MediaInfo_Open_Buffer_Continue,_MediaInfo_Open_Buffer_Continue_GoTo_Get,_MediaInfo_Open_Buffer_Finalize,_MediaInfo_Open,_MediaInfo_Close,_MediaInfo_Option,_MediaInfo_Inform,_MediaInfo_Count_Get,_MediaInfo_Get,_setlocale");
    println!("cargo:rustc-link-arg=-sSTANDALONE_WASM=1");
    
    // Don't import these functions from env - they should be resolved from static libs
    println!("cargo:rustc-link-arg=-sERROR_ON_UNDEFINED_SYMBOLS=0");
}
