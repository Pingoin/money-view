import 'package:app/generated/moneyview.pbgrpc.dart';
import 'package:english_words/english_words.dart';
import 'package:flutter/material.dart';
import 'package:grpc/grpc_web.dart';

class ApplicationState extends ChangeNotifier {
  var current = WordPair.random();

  var favorites = <WordPair>[];
  final moneyViewClient = MoneyViewClient(
      GrpcWebClientChannel.xhr(Uri.parse('http://localhost:50051')));

  void getNext() {
    current = WordPair.random();
    notifyListeners();
  }

  void toggleFavorite() {
    if (favorites.contains(current)) {
      favorites.remove(current);
    } else {
      favorites.add(current);
    }
    notifyListeners();
  }
}
