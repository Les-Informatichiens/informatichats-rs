use ffmpeg_sys_next::AVPixelFormat::AV_PIX_FMT_BGRA;
use ffmpeg_sys_next::{
    av_dict_set, av_find_best_stream, av_find_input_format, av_frame_alloc, av_inv_q,
    av_packet_alloc, av_packet_free, av_read_frame, avcodec_alloc_context3, avcodec_find_decoder,
    avcodec_find_encoder, avcodec_open2, avcodec_receive_frame, avcodec_send_packet,
    avdevice_register_all, avformat_alloc_context, avformat_find_stream_info,
    avformat_network_init, avformat_open_input, AVCodecContext, AVCodecID, AVDictionary,
    AVFormatContext, AVFrame, AVInputFormat, AVMediaType, AVPacket, AVPixelFormat,
    AVERROR_INVALIDDATA,
};
use std::ffi::{c_int, CString};

pub struct Context {
    encoder_ctx: *mut AVCodecContext,
    decoder_ctx: *mut AVCodecContext,
    av_frame: *mut AVFrame,
    input_ctx: *mut AVFormatContext,
}

impl Context {
    pub unsafe fn new(encoder_codec: AVCodecID, decoder_codec: AVCodecID) -> Self {
        unsafe {
            avdevice_register_all();
            avformat_network_init();
        }

        let mut input_ctx = avformat_alloc_context();
        if input_ctx.is_null() {
            panic!("Error allocating input context");
        }

        let input_format = find_screen_grab_input_format();
        let mut options: *mut AVDictionary = std::ptr::null_mut();
        if av_dict_set(
            &mut options,
            CString::new("probesize").unwrap().as_ptr(),
            CString::new("10000000").unwrap().as_ptr(), // Increase probesize
            0,
        ) != 0
        {
            panic!("Error setting probesize");
        }

        if av_dict_set(
            &mut options,
            CString::new("video_size").unwrap().as_ptr(),
            CString::new("1920x1080").unwrap().as_ptr(),
            0,
        ) != 0
        {
            panic!("Error setting video size");
        }
        if av_dict_set(
            &mut options,
            CString::new("framerate").unwrap().as_ptr(),
            CString::new("30").unwrap().as_ptr(),
            0,
        ) != 0
        {
            panic!("Error setting framerate");
        }

        if avformat_open_input(
            &mut input_ctx,
            CString::new(screen_grab_url()).unwrap().as_ptr(),
            input_format,
            &mut options,
        ) != 0
        {
            panic!("Error opening input device");
        }

        if avformat_find_stream_info(input_ctx, std::ptr::null_mut()) < 0 {
            panic!("Error finding stream info");
        }

        let encoder = avcodec_find_encoder(encoder_codec);
        if encoder.is_null() {
            panic!("Error finding encoder");
        }
        let encoder_ctx = avcodec_alloc_context3(encoder);
        if encoder_ctx.is_null() {
            panic!("Error allocating encoder context");
        }

        let video_stream = *(*input_ctx).streams.add(av_find_best_stream(
            input_ctx,
            AVMediaType::AVMEDIA_TYPE_VIDEO,
            -1,
            -1,
            std::ptr::null_mut(),
            0,
        ) as usize);

        println!(
            "video_stream pixel format: {:?}",
            std::mem::transmute::<c_int, AVPixelFormat>((*(*video_stream).codecpar).format)
        );
        println!("video_stream: {:?}", (*(*video_stream).codecpar).codec_id);

        let pix_fmt = AVPixelFormat::AV_PIX_FMT_BGRA;

        let codec_parameters = (*video_stream).codecpar;

        (*encoder_ctx).width = (*codec_parameters).width;
        (*encoder_ctx).height = (*codec_parameters).height;
        (*encoder_ctx).time_base = (*video_stream).time_base;
        (*encoder_ctx).framerate = av_inv_q((*video_stream).time_base);
        (*encoder_ctx).pix_fmt = pix_fmt;

        if avcodec_open2(encoder_ctx, encoder, std::ptr::null_mut()) < 0 {
            panic!("Error opening encoder");
        }

        let decoder = avcodec_find_decoder(decoder_codec);
        let decoder_ctx = avcodec_alloc_context3(decoder);
        if decoder_ctx.is_null() {
            panic!("Error allocating decoder context");
        }

        (*decoder_ctx).width = (*codec_parameters).width;
        (*decoder_ctx).height = (*codec_parameters).height;
        (*decoder_ctx).time_base = (*video_stream).time_base;
        (*decoder_ctx).framerate = av_inv_q((*video_stream).time_base);
        // (*decoder_ctx).pix_fmt = std::mem::transmute((*codec_parameters).format);
        (*decoder_ctx).pix_fmt = pix_fmt;

        if avcodec_open2(decoder_ctx, decoder, std::ptr::null_mut()) < 0 {
            panic!("Error opening decoder");
        }

        let av_frame = av_frame_alloc();
        if av_frame.is_null() {
            panic!("Error allocating frame");
        }

        (*av_frame).width = (*codec_parameters).width;
        (*av_frame).height = (*codec_parameters).height;
        // (*av_frame).format = (*codec_parameters).format;
        (*av_frame).format = AV_PIX_FMT_BGRA as i32;

        Context {
            encoder_ctx,
            decoder_ctx,
            av_frame,
            input_ctx,
        }
    }
}

impl Context {
    pub(crate) unsafe fn grab_screen(&self) {
        let mut pkt: *mut AVPacket = av_packet_alloc();
        if pkt.is_null() {
            panic!("Error allocating packet");
        }

        if av_read_frame(self.input_ctx, pkt) < 0 {
            panic!("Error reading frame");
        }

        let ret = avcodec_send_packet(self.decoder_ctx, pkt);
        if ret < 0 {
            if ret == AVERROR_INVALIDDATA {
                eprintln!("Invalid data found when processing input");
            } else {
                eprintln!("Error sending packet to decoder: {}", ret);
            }
            panic!("Error sending packet to decoder");
        }

        if avcodec_receive_frame(self.decoder_ctx, self.av_frame) < 0 {
            panic!("Error receiving frame from decoder");
        }

        std::fs::write(
            "frame.raw",
            std::slice::from_raw_parts(
                (*self.av_frame).data[0],
                (*self.av_frame).linesize[0] as usize * (*self.av_frame).height as usize,
            ),
        )
        .unwrap();

        av_packet_free(&mut pkt);
    }
}

fn find_screen_grab_input_format() -> *const AVInputFormat {
    unsafe {
        #[cfg(target_os = "linux")]
        let fmt_type = CString::new("x11grab").unwrap();
        #[cfg(target_os = "windows")]
        let fmt_type = CString::new("gdigrab").unwrap();

        av_find_input_format(fmt_type.as_ptr())
    }
}

fn screen_grab_url() -> String {
    #[cfg(target_os = "linux")]
    return String::from(":0.0");
    #[cfg(target_os = "windows")]
    return String::from("desktop");
}
