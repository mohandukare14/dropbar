import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:http/http.dart' as http;
import '../models/device_info.dart';

class DiscoveryService {
  static const int beaconPort = 53317;
  RawDatagramSocket? _socket;
  final StreamController<List<DeviceInfo>> _peersController =
      StreamController<List<DeviceInfo>>.broadcast();
  final Map<String, DeviceInfo> _peers = {};
  Timer? _cleanupTimer;
  Timer? _beaconTimer;
  Timer? _hotspotProbeTimer;

  Stream<List<DeviceInfo>> get peersStream => _peersController.stream;
  List<DeviceInfo> get currentPeers => _peers.values.toList();

  Future<void> start(DeviceInfo localDevice) async {
    try {
      _socket = await RawDatagramSocket.bind(InternetAddress.anyIPv4, beaconPort);
      _socket?.broadcastEnabled = true;

      _socket?.listen((event) {
        if (event == RawSocketEvent.read) {
          final datagram = _socket?.receive();
          if (datagram != null) {
            _handlePacket(datagram.data, datagram.address.address, localDevice.id);
          }
        }
      });

      // 1. Periodic broadcast beacon to multiple hotspot broadcast addresses
      _beaconTimer = Timer.periodic(const Duration(seconds: 3), (_) {
        _broadcastAnnounce(localDevice);
      });

      // 2. Active Hotspot Probe (Android Hotspot, iPhone Hotspot, Arch Hotspot)
      // When on a mobile hotspot, mobile queries standard gateway candidates directly
      _hotspotProbeTimer = Timer.periodic(const Duration(seconds: 4), (_) {
        _probeHotspotGateways(localDevice.id);
      });

      // 3. Cleanup peers not seen in 15 seconds
      _cleanupTimer = Timer.periodic(const Duration(seconds: 5), (_) {
        final now = DateTime.now();
        final initialCount = _peers.length;
        _peers.removeWhere((_, peer) =>
            now.difference(peer.lastSeen).inSeconds > 15);
        if (_peers.length != initialCount) {
          _peersController.add(_peers.values.toList());
        }
      });
    } catch (e) {
      print("Discovery start error: $e");
    }
  }

  void _broadcastAnnounce(DeviceInfo localDevice) {
    if (_socket == null) return;
    try {
      final msg = jsonEncode({
        "Announce": localDevice.toJson(),
      });
      final bytes = utf8.encode(msg);

      final targets = [
        "255.255.255.255",
        "192.168.43.255",
        "172.20.10.15",
        "172.20.10.255",
        "10.42.0.255",
      ];

      for (final ip in targets) {
        try {
          _socket?.send(bytes, InternetAddress(ip), beaconPort);
        } catch (_) {}
      }
    } catch (_) {}
  }

  Future<void> _probeHotspotGateways(String localId) async {
    final candidateGateways = [
      "10.42.0.1",     // Arch Linux NetworkManager Hotspot default gateway
      "192.168.12.1",  // Common Linux Hotspot gateway
      "192.168.43.1",  // Android hotspot host
      "172.20.10.1",   // iOS hotspot host
    ];

    for (final ip in candidateGateways) {
      try {
        final uri = Uri.parse("http://$ip:53318/api/v1/ping");
        final resp = await http.get(uri).timeout(const Duration(milliseconds: 600));
        if (resp.statusCode == 200) {
          final data = jsonDecode(resp.body) as Map<String, dynamic>;
          final peer = DeviceInfo.fromJson(data);
          if (peer.id != localId) {
            final verifiedPeer = DeviceInfo(
              id: peer.id,
              name: peer.name,
              deviceType: peer.deviceType,
              ip: ip,
              port: peer.port,
              lastSeen: DateTime.now(),
            );
            _peers[peer.id] = verifiedPeer;
            _peersController.add(_peers.values.toList());
          }
        }
      } catch (_) {
        // Expected when candidate IP is unreachable
      }
    }
  }

  void _handlePacket(List<int> bytes, String remoteIp, String localId) {
    try {
      final str = utf8.decode(bytes);
      final json = jsonDecode(str) as Map<String, dynamic>;
      if (json.containsKey("Announce")) {
        final peer = DeviceInfo.fromJson(json["Announce"] as Map<String, dynamic>);
        if (peer.id != localId) {
          final peerWithIp = DeviceInfo(
            id: peer.id,
            name: peer.name,
            deviceType: peer.deviceType,
            ip: remoteIp,
            port: peer.port,
            lastSeen: DateTime.now(),
          );
          _peers[peer.id] = peerWithIp;
          _peersController.add(_peers.values.toList());
        }
      }
    } catch (_) {}
  }

  void addManualPeer(String ip, int port, String name) {
    final manual = DeviceInfo(
      id: "manual_$ip",
      name: name,
      deviceType: "ArchLinux",
      ip: ip,
      port: port,
      lastSeen: DateTime.now(),
    );
    _peers[manual.id] = manual;
    _peersController.add(_peers.values.toList());
  }

  void stop() {
    _cleanupTimer?.cancel();
    _beaconTimer?.cancel();
    _hotspotProbeTimer?.cancel();
    _socket?.close();
    _peersController.close();
  }
}
