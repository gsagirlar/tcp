use etherparse;
use std::io;

pub enum State {
    Closed,
    Listen,
    SynRcvd,
    //Estab,
}

//#[derive(Debug)]
pub struct Connection {
    state: State,
    recv: ReceiveSequence,
    send: SendSequence,
}

pub struct SendSequence {
    // send unacknowledge
    una: u32,
    // send next
    nxt: u32,
    // send window
    wnd: u16,
    // send urgent pointer
    up: bool,
    // seqment sequence number used for last window update
    wl1: usize,
    // seqment acknowledgement number used for last window update
    wl2: usize,
    // inital send sequence number
    iss: u32,
}

pub struct ReceiveSequence {
    // send next
    nxt: u32,
    // send window
    wnd: u16,
    // send urgent pointer
    up: bool,
    // inital receive sequence number
    irs: u32,
}

// impl Default for Connection {
// 	fn default() -> Self {
// 		//State::Closed
// 		Connection {
// 			state: State::Listen,
// 			recv: ReceiveSequence{},
// 			send: SendSequence{},
// 		}
// 	}
// }

impl Connection {
    //returns a new connection
    pub fn accept<'a>(
        nic: &mut tun_tap::Iface,
        ip_header: etherparse::Ipv4HeaderSlice<'a>,
        tcp_header: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) -> io::Result<Option<Connection>> {
        let mut buff = [0u8; 1500];

        if !tcp_header.syn() {
            //only expected SYN packet
            return Ok(None);
        }
        let iss = 0;
        let mut c = Connection {
            state: State::SynRcvd,
            recv: ReceiveSequence {
                //keep track of sender info
                irs: tcp_header.sequence_number(),
                nxt: tcp_header.sequence_number() + 1,
                wnd: tcp_header.window_size(),
                up: false,
                //irs is the sequence number in received TCP packet
            },

            send: SendSequence {
                //establish sending info
                //we are initializing the sending operation, starting from zero
                //iss is the sequence number of the Syn
                iss,
                una: iss,
                nxt: iss + 1,
                wnd: 10,

                up: false,
                wl1: 0,
                wl2: 0,
            },
        };

        //establish connection, reply with Syn Ack
        let mut syn_ack = etherparse::TcpHeader::new(
            tcp_header.destination_port(),
            tcp_header.source_port(),
            c.send.iss,
            c.send.wnd,
        );

        syn_ack.acknowledgment_number = c.recv.nxt;
        syn_ack.syn = true;
        syn_ack.ack = true;

        let mut ip = etherparse::Ipv4Header::new(
            syn_ack.header_len(),
            64,
            etherparse::IpNumber::Tcp as u8,
            [
                ip_header.destination()[0],
                ip_header.destination()[1],
                ip_header.destination()[2],
                ip_header.destination()[3],
            ],
            [
                ip_header.source()[0],
                ip_header.source()[1],
                ip_header.source()[2],
                ip_header.source()[3],
            ],
        );
        //use entrie buffer to write, and find how much spaces haven't been used yet
        let mut unwritten = {
            let mut unwritten = &mut buff[..];
            ip.write(&mut unwritten);
            syn_ack.write(&mut unwritten);
            //used to calculate how much space is remaining in the buffer
            unwritten.len()
        };
        nic.send(&buff[..unwritten]);
        return Ok(Some(c));
    }

    pub fn on_packet(&mut self) {}
}
