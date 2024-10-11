import 'package:app/generated/moneyview.pbgrpc.dart';
import 'package:english_words/english_words.dart';
import 'package:flutter/material.dart';

import 'grpc_channel.dart'
    if (dart.library.html) 'grpc_channel_web.dart';



class ApplicationState extends ChangeNotifier {
  var current = WordPair.random();

  var favorites = <WordPair>[];



  dynamic moneyViewClient = MoneyViewClient(
      getChannel());

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