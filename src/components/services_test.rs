#[cfg(test)]
mod tests {
    use crate::components::{
        dto::{CreateOrderRequest, OrderSide, OrderStatus, OrderType, TimeInForce},
        services::OrderBookService,
    };
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use uuid::Uuid;

    #[test]
    fn should_add_order() {
        let mut order_book = OrderBookService::new();
        let create_order_request = CreateOrderRequest {
            item_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: Decimal::from_str("10.0").unwrap(),
            time_in_force: TimeInForce::DAY,
            quantity: Decimal::from_str("100.0").unwrap(),
        };
        let order = order_book.add_order(create_order_request).unwrap();
        assert_eq!(order.quantity, Decimal::from_str("100.0").unwrap());
        assert_eq!(matches!(order.order_side, OrderSide::Buy), true);
        assert_eq!(matches!(order.status, OrderStatus::Open), true);
    }

    #[test]
    fn should_get_order_by_id() {
        let mut order_book = OrderBookService::new();
        let create_order_request = CreateOrderRequest {
            item_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Sell,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::DAY,
            price: Decimal::from_str("20.0").unwrap(),
            quantity: Decimal::from_str("50.0").unwrap(),
        };
        let order = order_book.add_order(create_order_request).unwrap();
        let fetched_order = order_book.get_order_by_id(order.id);
        assert!(fetched_order.is_some());
        assert_eq!(fetched_order.unwrap().id, order.id);
    }

    #[test]
    fn should_update_order_status() {
        let mut order_book = OrderBookService::new();
        let create_order_request = CreateOrderRequest {
            item_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Buy,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::DAY,
            price: Decimal::from_str("15.0").unwrap(),
            quantity: Decimal::from_str("100.0").unwrap(),
        };
        let order = order_book.add_order(create_order_request).unwrap();
        let updated_order = order_book.update_order_status(order.id, OrderStatus::Closed);
        assert!(updated_order.is_some());
        assert_eq!(
            matches!(updated_order.unwrap().status, OrderStatus::Closed),
            true
        );
    }

    #[test]
    fn should_update_order_quantity() {
        let mut order_book = OrderBookService::new();
        let create_order_request = CreateOrderRequest {
            item_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Sell,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::DAY,
            price: Decimal::from_str("25.0").unwrap(),
            quantity: Decimal::from_str("50.0").unwrap(),
        };
        let order = order_book.add_order(create_order_request).unwrap();
        let updated_order =
            order_book.update_order_quantity(order.id, Decimal::from_str("75.0").unwrap());
        assert!(updated_order.is_some());
        assert_eq!(
            updated_order.unwrap().quantity,
            Decimal::from_str("75.0").unwrap()
        );
    }

    #[test]
    fn should_cancel_order() {
        let mut order_book = OrderBookService::new();
        let create_order_request = CreateOrderRequest {
            item_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Buy,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::DAY,
            price: Decimal::from_str("30.0").unwrap(),
            quantity: Decimal::from_str("100.0").unwrap(),
        };
        let order = order_book.add_order(create_order_request).unwrap();
        order_book.cancel_order(order.id);
        let fetched_order = order_book.get_order_by_id(order.id);
        assert!(matches!(
            fetched_order.unwrap().status,
            OrderStatus::Cancelled
        ));
    }

    #[test]
    fn should_be_partially_filled() {
        let mut order_book = OrderBookService::new();
        let item_id = Uuid::new_v4();

        let buy_order_request = CreateOrderRequest {
            item_id,
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Buy,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::DAY,
            price: Decimal::from_str("10.0").unwrap(),
            quantity: Decimal::from_str("100.0").unwrap(),
        };
        let buy_order = order_book.add_order(buy_order_request).unwrap();

        let sell_order_request = CreateOrderRequest {
            item_id,
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Sell,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::DAY,
            price: Decimal::from_str("10.0").unwrap(),
            quantity: Decimal::from_str("50.0").unwrap(),
        };
        let sell_order = order_book.add_order(sell_order_request).unwrap();

        let fetched_buy_order = order_book.get_order_by_id(buy_order.id).unwrap();
        let fetched_sell_order = order_book.get_order_by_id(sell_order.id).unwrap();

        assert_eq!(
            fetched_buy_order.quantity_filled,
            Decimal::from_str("50.0").unwrap()
        );
        assert_eq!(
            fetched_sell_order.quantity_filled,
            Decimal::from_str("50.0").unwrap()
        );
        assert_eq!(
            matches!(fetched_buy_order.status, OrderStatus::PartiallyFilled),
            true
        );
        assert_eq!(
            matches!(fetched_sell_order.status, OrderStatus::Closed),
            true
        );
    }

    #[test]
    fn should_fully_filled() {
        let mut order_book = OrderBookService::new();
        let item_id = Uuid::new_v4();

        let buy_order_request = CreateOrderRequest {
            item_id,
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Buy,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::DAY,
            price: Decimal::from_str("10.0").unwrap(),
            quantity: Decimal::from_str("100.0").unwrap(),
        };
        let buy_order = order_book.add_order(buy_order_request).unwrap();

        let sell_order_request = CreateOrderRequest {
            item_id,
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Sell,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::DAY,
            price: Decimal::from_str("10.0").unwrap(),
            quantity: Decimal::from_str("100.0").unwrap(),
        };
        let sell_order = order_book.add_order(sell_order_request).unwrap();

        let fetched_buy_order = order_book.get_order_by_id(buy_order.id).unwrap();
        let fetched_sell_order = order_book.get_order_by_id(sell_order.id).unwrap();

        assert_eq!(
            fetched_buy_order.quantity_filled,
            Decimal::from_str("100.0").unwrap()
        );
        assert_eq!(
            fetched_sell_order.quantity_filled,
            Decimal::from_str("100.0").unwrap()
        );
        assert_eq!(
            matches!(fetched_buy_order.status, OrderStatus::Closed),
            true
        );
        assert_eq!(
            matches!(fetched_sell_order.status, OrderStatus::Closed),
            true
        );
    }

    #[test]
    fn should_update_order_price() {
        let mut order_book = OrderBookService::new();
        let create_order_request = CreateOrderRequest {
            item_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Buy,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::DAY,
            price: Decimal::from_str("10.0").unwrap(),
            quantity: Decimal::from_str("100.0").unwrap(),
        };
        let order = order_book.add_order(create_order_request).unwrap();
        let updated_order =
            order_book.update_order_price(order.id, Decimal::from_str("15.0").unwrap());
        assert!(updated_order.is_some());
        assert_eq!(
            updated_order.unwrap().price,
            Decimal::from_str("15.0").unwrap()
        );
    }

    #[test]
    fn trades_should_contain_filled_orders() {
        let mut order_book = OrderBookService::new();
        let item_id = Uuid::new_v4();

        let buy_order_request = CreateOrderRequest {
            item_id,
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Buy,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::DAY,
            price: Decimal::from_str("10.0").unwrap(),
            quantity: Decimal::from_str("100.0").unwrap(),
        };
        let buy_order = order_book.add_order(buy_order_request).unwrap();

        let sell_order_request = CreateOrderRequest {
            item_id,
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Sell,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::DAY,
            price: Decimal::from_str("10.0").unwrap(),
            quantity: Decimal::from_str("50.0").unwrap(),
        };
        let sell_order = order_book.add_order(sell_order_request).unwrap();

        assert_eq!(order_book.trades.len(), 1);
        let trade = &order_book.trades[0];
        assert_eq!(trade.buy_order_id, buy_order.id);
        assert_eq!(trade.sell_order_id, sell_order.id);
    }

    #[test]
    fn should_update_order_quantity_and_price() {
        let mut order_book = OrderBookService::new();
        let create_order_request = CreateOrderRequest {
            item_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Sell,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::DAY,
            price: Decimal::from_str("20.0").unwrap(),
            quantity: Decimal::from_str("50.0").unwrap(),
        };
        let order = order_book.add_order(create_order_request).unwrap();
        let updated_order =
            order_book.update_order_quantity(order.id, Decimal::from_str("75.0").unwrap());
        assert!(updated_order.is_some());
        assert_eq!(
            updated_order.unwrap().quantity,
            Decimal::from_str("75.0").unwrap()
        );

        let updated_order_price =
            order_book.update_order_price(order.id, Decimal::from_str("25.0").unwrap());
        assert!(updated_order_price.is_some());
        assert_eq!(
            updated_order_price.unwrap().price,
            Decimal::from_str("25.0").unwrap()
        );
    }

    #[test]
    fn should_not_match_orders_with_incompatible_prices() {
        let mut order_book = OrderBookService::new();
        let item_id = Uuid::new_v4();
        let buy_order_request = CreateOrderRequest {
            item_id,
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Buy,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::DAY,
            price: Decimal::from_str("10.0").unwrap(),
            quantity: Decimal::from_str("100.0").unwrap(),
        };

        let buy_order = order_book.add_order(buy_order_request).unwrap();
        let sell_order_request = CreateOrderRequest {
            item_id,
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Sell,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::DAY,
            price: Decimal::from_str("15.0").unwrap(),
            quantity: Decimal::from_str("50.0").unwrap(),
        };

        let sell_order = order_book.add_order(sell_order_request).unwrap();
        let fetched_buy_order = order_book.get_order_by_id(buy_order.id).unwrap();
        let fetched_sell_order = order_book.get_order_by_id(sell_order.id).unwrap();

        assert_eq!(fetched_buy_order.quantity_filled, Decimal::ZERO);
        assert_eq!(fetched_sell_order.quantity_filled, Decimal::ZERO);
        assert_eq!(matches!(fetched_buy_order.status, OrderStatus::Open), true);
        assert_eq!(matches!(fetched_sell_order.status, OrderStatus::Open), true);
    }

    #[test]
    fn should_error_market_order_without_existing_orders() {
        let mut order_book = OrderBookService::new();
        let create_order_request = CreateOrderRequest {
            item_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Buy,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::DAY,
            price: Decimal::ZERO,
            quantity: Decimal::from_str("100.0").unwrap(),
        };
        let result = order_book.add_order(create_order_request);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            "Market order cannot be placed without any existing orders to determine price"
        );
    }

    #[test]
    fn should_fill_market_order_with_existing_orders() {
        let mut order_book = OrderBookService::new();
        let item_id = Uuid::new_v4();

        let sell_order_request = CreateOrderRequest {
            item_id,
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Sell,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::DAY,
            price: Decimal::from_str("10.0").unwrap(),
            quantity: Decimal::from_str("50.0").unwrap(),
        };
        let _ = order_book.add_order(sell_order_request);
        let current_market_price = order_book
            .get_current_market_price(item_id, OrderSide::Buy)
            .unwrap();

        let buy_market_order_request = CreateOrderRequest {
            item_id,
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Buy,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::DAY,
            price: current_market_price,
            quantity: Decimal::from_str("50.0").unwrap(),
        };
        let buy_market_order = order_book.add_order(buy_market_order_request).unwrap();

        assert_eq!(buy_market_order.price, Decimal::from_str("10.0").unwrap());
        assert_eq!(
            buy_market_order.quantity_filled,
            Decimal::from_str("50.0").unwrap()
        );
    }

    #[test]
    fn should_partially_fill_ioc_order() {
        let mut order_book = OrderBookService::new();
        let item_id = Uuid::new_v4();

        let sell_order_request = CreateOrderRequest {
            item_id,
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Sell,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::DAY,
            price: Decimal::from_str("10.0").unwrap(),
            quantity: Decimal::from_str("50.0").unwrap(),
        };
        let _ = order_book.add_order(sell_order_request);

        let buy_ioc_order_request = CreateOrderRequest {
            item_id,
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Buy,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::IOC,
            price: Decimal::from_str("10.0").unwrap(),
            quantity: Decimal::from_str("100.0").unwrap(),
        };
        let buy_ioc_order = order_book.add_order(buy_ioc_order_request).unwrap();
        assert_eq!(
            buy_ioc_order.quantity_filled,
            Decimal::from_str("50.0").unwrap()
        );
        assert_eq!(buy_ioc_order.quantity, Decimal::from_str("50.0").unwrap());
        assert_eq!(matches!(buy_ioc_order.status, OrderStatus::Closed), true);
    }

    #[test]
    fn should_not_fill_because_invalid_market_price() {
        let mut order_book = OrderBookService::new();
        let item_id = Uuid::new_v4();

        let sell_order_request = CreateOrderRequest {
            item_id,
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Sell,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::DAY,
            price: Decimal::from_str("30.0").unwrap(),
            quantity: Decimal::from_str("50.0").unwrap(),
        };
        let _ = order_book.add_order(sell_order_request);

        let buy_market_order_request = CreateOrderRequest {
            item_id,
            user_id: Uuid::new_v4(),
            order_side: OrderSide::Buy,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::DAY,
            price: Decimal::from_str("20.0").unwrap(),
            quantity: Decimal::from_str("50.0").unwrap(),
        };
        let result = order_book.add_order(buy_market_order_request);
        assert!(result.is_err());

        let err_msg = result.err().unwrap();
        assert!(err_msg.contains("Market order price cannot be more than 5% away"));
        assert!(err_msg.contains("30"));
        assert!(err_msg.contains("20"));
    }
}
